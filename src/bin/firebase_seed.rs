// firebase_seed — one-off dev utility: populate Firestore `cv` collection (Phase 6)
//
// Requires write access to Firestore (your user account or a writer service account).
// Read-only `cv-reader` credentials for the pipeline are created separately — see
// firebase-seed/README.md and scripts/firebase/create-cv-reader.ps1
//
//   gcloud auth login
//   gcloud config set project battleplan-dev-2024
//   cargo run --bin firebase_seed
//
//   cargo run --bin firebase_seed -- --dry-run

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde_json::{json, Value};

const DEFAULT_PROJECT: &str = "battleplan-dev-2024";
const COLLECTION: &str = "cv";

struct Args {
    dry_run: bool,
    project: String,
}

fn parse_args() -> Args {
    let mut dry_run = false;
    let mut project = DEFAULT_PROJECT.to_string();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--project" => {
                project = args.next().expect("--project requires a value");
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_help();
                std::process::exit(1);
            }
        }
    }
    Args { dry_run, project }
}

fn print_help() {
    eprintln!(
        "firebase_seed — populate Firestore cv/{{availability,skills,highlights}}

Usage:
  cargo run --bin firebase_seed [-- --dry-run] [-- --project PROJECT]

Auth (first match wins):
  1. gcloud auth print-access-token  (gcloud auth login + project set)
  2. GOOGLE_APPLICATION_CREDENTIALS  (service account JSON path)

Data files: firebase-seed/data/*.json (relative to repo root)"
    );
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = parse_args();
    let data_dir = data_dir()?;

    let documents = [
        ("availability", data_dir.join("availability.json")),
        ("skills", data_dir.join("skills.json")),
        ("highlights", data_dir.join("highlights.json")),
    ];

    if args.dry_run {
        println!("dry-run — project={}", args.project);
        for (doc_id, path) in &documents {
            let raw: Value = read_json(path)?;
            println!("\n--- cv/{doc_id} ({}) ---", path.display());
            println!("{}", serde_json::to_string_pretty(&raw)?);
            let fields = json_to_firestore_fields(&raw)?;
            println!("Firestore fields: {}", serde_json::to_string_pretty(&fields)?);
        }
        return Ok(());
    }

    let token = access_token().await?;
    let client = reqwest::Client::new();

    for (doc_id, path) in documents {
        let raw = read_json(&path)?;
        upsert_document(&client, &token, &args.project, doc_id, &raw).await?;
        println!("✓  cv/{doc_id}");
    }

    println!("\nDone. Create read-only pipeline credentials:");
    println!("  .\\scripts\\firebase\\create-cv-reader.ps1   (Windows)");
    println!("  ./scripts/firebase/create-cv-reader.sh      (bash/WSL)");
    Ok(())
}

fn data_dir() -> anyhow::Result<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = manifest.join("firebase-seed").join("data");
    if dir.is_dir() {
        return Ok(dir);
    }
    // Fallback when cwd is repo root but compile path differs
    let cwd = std::env::current_dir()?.join("firebase-seed").join("data");
    anyhow::ensure!(cwd.is_dir(), "firebase-seed/data not found (run from repo root)");
    Ok(cwd)
}

fn read_json(path: &Path) -> anyhow::Result<Value> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&text)?)
}

async fn upsert_document(
    client: &reqwest::Client,
    token: &str,
    project: &str,
    doc_id: &str,
    raw: &Value,
) -> anyhow::Result<()> {
    let fields = json_to_firestore_fields(raw)?;
    let url = format!(
        "https://firestore.googleapis.com/v1/projects/{project}/databases/(default)/documents/{COLLECTION}/{doc_id}"
    );

    let mask: Vec<String> = raw
        .as_object()
        .context("document JSON must be a top-level object")?
        .keys()
        .cloned()
        .collect();

    let query: Vec<(&str, &str)> = mask
        .iter()
        .map(|k| ("updateMask.fieldPaths", k.as_str()))
        .collect();

    let resp = client
        .patch(&url)
        .bearer_auth(token)
        .query(&query)
        .json(&json!({ "fields": fields }))
        .send()
        .await?
        .error_for_status()
        .with_context(|| format!("PATCH cv/{doc_id}"))?;

    let _body: Value = resp.json().await?;
    Ok(())
}

async fn access_token() -> anyhow::Result<String> {
    if let Ok(path) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        return service_account_token(&path).await;
    }
    gcloud_access_token()
}

fn gcloud_access_token() -> anyhow::Result<String> {
    // Windows ships gcloud as gcloud.cmd; Rust's Command does not resolve bare "gcloud".
    #[cfg(windows)]
    const GCLOUD: &str = "gcloud.cmd";
    #[cfg(not(windows))]
    const GCLOUD: &str = "gcloud";

    let output = std::process::Command::new(GCLOUD)
        .args(["auth", "print-access-token"])
        .output()
        .context("failed to run `gcloud auth print-access-token` — install gcloud SDK and run `gcloud auth login`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gcloud auth print-access-token failed: {stderr}");
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

async fn service_account_token(path: &str) -> anyhow::Result<String> {
    use gcp_auth::TokenProvider as _;
    let json = std::fs::read_to_string(path)?;
    let sa = gcp_auth::CustomServiceAccount::from_json(&json)
        .map_err(|e| anyhow::anyhow!("invalid service account JSON: {e}"))?;
    let scopes = &["https://www.googleapis.com/auth/datastore"];
    let token = sa.token(scopes).await
        .map_err(|e| anyhow::anyhow!("failed to obtain GCP token: {e}"))?;
    Ok(token.as_str().to_string())
}

/// Convert plain JSON into Firestore REST `fields` map.
fn json_to_firestore_fields(value: &Value) -> anyhow::Result<HashMap<String, Value>> {
    let obj = value
        .as_object()
        .context("top-level document must be a JSON object")?;
    let mut out = HashMap::new();
    for (k, v) in obj {
        out.insert(k.clone(), json_to_firestore_value(v)?);
    }
    Ok(out)
}

fn json_to_firestore_value(value: &Value) -> anyhow::Result<Value> {
    Ok(match value {
        Value::Null => json!({ "nullValue": null }),
        Value::Bool(b) => json!({ "booleanValue": b }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                json!({ "integerValue": i.to_string() })
            } else if let Some(u) = n.as_u64() {
                json!({ "integerValue": u.to_string() })
            } else {
                json!({ "doubleValue": n.as_f64().unwrap_or(0.0) })
            }
        }
        Value::String(s) => json!({ "stringValue": s }),
        Value::Array(items) => {
            let values: Vec<Value> = items
                .iter()
                .map(json_to_firestore_value)
                .collect::<anyhow::Result<_>>()?;
            json!({ "arrayValue": { "values": values } })
        }
        Value::Object(map) => {
            let mut fields = HashMap::new();
            for (k, v) in map {
                fields.insert(k.clone(), json_to_firestore_value(v)?);
            }
            json!({ "mapValue": { "fields": fields } })
        }
    })
}
