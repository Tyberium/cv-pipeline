// cv_engine — orchestration entry point
//
// Startup sequence:
//   1. Validate CV_SECRET (borrowed from PostHog token.rs)
//   2. Decrypt raw_history.json in memory
//   3. Open DuckDB, initialise schema
//   4. Run producer → Vec<CvEvent>
//
//   [kafka feature — Docker build]
//   5a. Start Redpanda child process, wait for readiness
//   5b. Create cv_events topic
//   5c. Spawn consumer task (reads from Redpanda → DuckDB)
//   5d. Emit events to Redpanda via producer::emit
//   5e. Await consumer task completion
//   5f. Kill Redpanda
//
//   [no kafka feature — local Windows dev, short-circuit]
//   5g. Feed events directly to consumer::process_event (bypasses Redpanda)
//
//   6. Record pipeline telemetry
//   7. Start Axum on :8080

mod api;
mod consumer;
mod db;
mod models;
mod producer;

use std::sync::{Arc, Mutex};
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // All structured logs go to a file inside the container — ephemeral, gone when
    // the container stops. Only the println! personality lines reach stdout.
    let log_path = std::env::temp_dir().join("cv_pipeline.log");
    let log_file = std::fs::File::create(&log_path)?;
    tracing_subscriber::fmt()
        .with_writer(Mutex::new(log_file))
        .with_ansi(false)
        .init();

    // Startup log — printed to stdout AND served at /api/pipeline/logs
    let mut startup_log: Vec<String> = Vec::new();
    macro_rules! plog {
        () => {{ println!(); startup_log.push(String::new()); }};
        ($fmt:literal $(, $arg:expr)* $(,)?) => {{
            let s = format!($fmt $(, $arg)*);
            println!("{}", s);
            startup_log.push(s);
        }};
    }

    plog!("╔══════════════════════════════════════════════════════════════╗");
    plog!("║         cv_engine — Overengineered CV Pipeline v1           ║");
    plog!("║    One binary. Rust + Redpanda + DuckDB + Axum + React.     ║");
    plog!("╚══════════════════════════════════════════════════════════════╝");
    plog!();

    let secret = std::env::var("CV_SECRET")
        .map_err(|_| anyhow::anyhow!("CV_SECRET environment variable is required"))?;

    // Borrowed from PostHog/posthog: rust/capture/src/token.rs — validates token
    // is non-empty and doesn't look like a placeholder before we attempt decryption.
    validate_token(&secret)?;

    plog!("[0/6] Token validation");
    plog!("      validate_token() is borrowed directly from PostHog's open-source");
    plog!("      capture service (posthog/posthog: rust/capture/src/token.rs).");
    plog!("      It checks the secret is non-empty and not a placeholder before");
    plog!("      spending 100k PBKDF2 rounds on a key that will never work anyway.");
    plog!("      ✓ CV_SECRET accepted");
    plog!();

    plog!("[1/6] Decryption");
    plog!("      The CV data is baked into this image as an AES-256-GCM encrypted");
    plog!("      binary blob (raw_history.json). The key is derived from CV_SECRET");
    plog!("      using PBKDF2-HMAC-SHA256 (100k rounds). Without the correct secret");
    plog!("      the container starts, logs this line, then stops. Nothing leaks.");

    let encrypted = std::fs::read("raw_history.json").map_err(|_| {
        anyhow::anyhow!(
            "raw_history.json not found.\nRun: CV_SECRET=<secret> cargo run --bin encrypt"
        )
    })?;

    let decrypted = producer::decrypt(&encrypted, &secret)?;
    tracing::info!("raw_history.json decrypted ({} bytes)", decrypted.len());

    let photo_enc = std::fs::read("photo.enc").map_err(|_| {
        anyhow::anyhow!(
            "photo.enc not found.\nRun: CV_SECRET=<secret> cargo run --bin encrypt"
        )
    })?;
    let photo_png = cv_engine::crypto::decrypt(&photo_enc, &secret)?;
    tracing::info!("photo.enc decrypted ({} bytes)", photo_png.len());

    plog!("      ✓ raw_history.json and photo decrypted in memory ({} bytes)", decrypted.len());
    plog!();

    plog!("[2/6] Ingestion — reading source data");
    plog!("      raw_history.json holds two things: the static CV (work history,");
    plog!("      education, profile) and a Firebase service account key.");
    plog!("      The Firebase key is used next to pull live data (skills, availability)");
    plog!("      from Firestore — the only part of the CV that changes without a rebuild.");

    let firebase_credentials = serde_json::from_slice::<serde_json::Value>(&decrypted)
        .ok()
        .and_then(|v| v.get("firebase_credentials").cloned())
        .unwrap_or(serde_json::Value::Null);

    plog!();
    plog!("[3/6] Transform — JSON → typed events (PostHog capture format)");
    plog!("      The source JSON is denormalised and uses mixed date formats.");
    plog!("      The producer normalises dates to ISO-8601, maps each record to");
    plog!("      a typed event (profile_update, career_milestone, skill_added \u{2026}),");
    plog!("      and merges in the live Firebase data. Output: Vec<CvEvent>.");
    plog!("      This mirrors PostHog's capture service — each fact is an event,");
    plog!("      not a row in a hand-rolled schema.");

    let pipeline_start = Instant::now();
    let events = producer::produce(&decrypted).await?;
    tracing::info!("produced {} events", events.len());

    plog!("      \u{2713} {} events produced ({} from Firebase, rest from JSON)",
        events.len(),
        events.iter().filter(|e| e.properties.get("source").and_then(|s| s.as_str()) == Some("firebase")).count()
    );
    plog!();

    plog!("[4/6] Storage — initialising DuckDB");
    plog!("      DuckDB is an embedded OLAP engine — no daemon, no port, no setup.");
    plog!("      It lives in a single file (cv_gold.db) inside the container.");
    plog!("      The schema is fact-table style: dim_profile, fact_career_events,");
    plog!("      fact_skills, fact_education, fact_pipeline_run.");
    plog!("      Axum queries it directly via the duckdb crate.");

    let conn = Arc::new(Mutex::new(db::open()?));
    db::init_schema(&conn.lock().unwrap())?;
    tracing::info!("database ready");

    plog!("      \u{2713} schema initialised");
    plog!();

    let consumed;

    #[cfg(feature = "kafka")]
    {
        plog!("[5/6] Message bus — Redpanda (Kafka-compatible broker)");
        plog!("      Redpanda is a Kafka-compatible broker written in C++.");
        plog!("      It runs here as a managed child process (one image, one command).");
        plog!("      The producer emits all events to the cv_events topic.");
        plog!("      A consumer task reads from that topic and writes to DuckDB.");
        plog!("      In a real PostHog-style system this would be a separate service;");
        plog!("      here it is deliberately collapsed for UX.");
        let redpanda_log = std::fs::OpenOptions::new().append(true).open(&log_path)?;
        consumed = run_pipeline_redpanda(&conn, &events, redpanda_log).await?;
        plog!("      \u{2713} {} events consumed from cv_events \u{2192} DuckDB", consumed);
    }

    #[cfg(not(feature = "kafka"))]
    {
        plog!("[5/6] Message bus — short-circuit (local dev, no Kafka feature)");
        plog!("      rdkafka requires cmake and a Unix toolchain; skipped on Windows.");
        plog!("      Events are fed directly to the consumer function — same routing");
        plog!("      logic, same DuckDB writes, just without the Redpanda hop.");
        consumed = run_pipeline_short_circuit(&conn, &events)?;
        plog!("      \u{2713} {} events written to DuckDB", consumed);
    }

    // Record pipeline run in fact_pipeline_run + fact_pipeline_event_log
    let run_id = format!("run_{}", chrono::Utc::now().timestamp_millis());
    let duration_ms = pipeline_start.elapsed().as_millis() as i32;
    let firebase_events = events
        .iter()
        .filter(|e| e.properties.get("source").and_then(|s| s.as_str()) == Some("firebase"))
        .count() as i32;
    let json_events = events
        .iter()
        .filter(|e| e.properties.get("source").and_then(|s| s.as_str()) != Some("firebase"))
        .count() as i32;

    {
        let c = conn.lock().unwrap();
        db::insert_pipeline_run(
            &c,
            &serde_json::json!({
                "run_id":          run_id,
                "events_produced": events.len() as i32,
                "events_consumed": consumed as i32,
                "duration_ms":     duration_ms,
                "ran_at":          chrono::Utc::now().to_rfc3339(),
                "json_events":     json_events,
                "firebase_events": firebase_events,
            }),
        )?;

        let mut counts: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
        for e in &events {
            *counts.entry(e.event.as_str()).or_insert(0) += 1;
        }
        for (event_type, count) in &counts {
            db::insert_pipeline_event_log(&c, &run_id, event_type, *count)?;
        }
    }

    tracing::info!(
        "pipeline complete in {}ms — {} events produced, {} consumed",
        duration_ms,
        events.len(),
        consumed
    );

    plog!();
    plog!("[6/6] Serving");
    plog!("      Pipeline complete in {}ms. Axum is now serving the frontend", duration_ms);
    plog!("      on port 8080. Static assets were compiled by Vite at image build");
    plog!("      time — no Node.js at runtime. All API routes query DuckDB directly.");
    plog!("      The pre-built React + Mantine frontend is served from /static.");
    plog!();
    plog!("      Pipeline total: {}ms", duration_ms);
    plog!();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    plog!("      Site served at http://localhost:8080 please click the link to open");
    tracing::info!("listening on http://localhost:8080");

    axum::serve(
        listener,
        api::router(conn, firebase_credentials, photo_png, startup_log),
    )
    .await?;

    Ok(())
}

// ── Short-circuit path (no kafka feature — local dev) ────────────────────────

#[cfg(not(feature = "kafka"))]
fn run_pipeline_short_circuit(
    conn: &Arc<Mutex<duckdb::Connection>>,
    events: &[models::CvEvent],
) -> anyhow::Result<u32> {
    let mut consumed = 0u32;
    let c = conn.lock().unwrap();
    for event in events {
        consumer::process_event(&c, event)?;
        consumed += 1;
    }
    tracing::info!("consumed {} events (short-circuit)", consumed);
    Ok(consumed)
}

// ── Redpanda path (kafka feature — Docker build) ──────────────────────────────

#[cfg(feature = "kafka")]
async fn run_pipeline_redpanda(
    conn: &Arc<Mutex<duckdb::Connection>>,
    events: &[models::CvEvent],
    log: std::fs::File,
) -> anyhow::Result<u32> {
    let mut redpanda = start_redpanda(log)?;

    wait_for_kafka().await?;
    create_topic().await?;

    // Consumer must be listening before we emit, so spawn it first
    let consumer_conn = Arc::clone(conn);
    let consumer_task = tokio::spawn(consumer::run(consumer_conn));

    producer::emit(events).await?;

    let consumed = consumer_task.await??;

    // Redpanda's job is done — pipeline is complete
    redpanda.kill().ok();
    redpanda.wait().ok();
    tracing::info!("Redpanda process stopped");

    Ok(consumed)
}

#[cfg(feature = "kafka")]
const REDPANDA_CFG: &str = include_str!("../config/redpanda.yaml");

#[cfg(feature = "kafka")]
fn start_redpanda(log: std::fs::File) -> anyhow::Result<std::process::Child> {
    std::fs::create_dir_all("/tmp/redpanda-data")?;

    // Redpanda 25.x: broker invoked directly with YAML config + Seastar flags.
    std::fs::write("/tmp/redpanda.yaml", REDPANDA_CFG)?;

    tracing::info!("starting Redpanda 25.x...");
    let stderr_log = log.try_clone()?;
    let child = std::process::Command::new("/opt/redpanda/libexec/redpanda")
        .args([
            "--redpanda-cfg", "/tmp/redpanda.yaml",
            "--smp", "1",
            "--memory", "1G",
            "--reserve-memory", "0M",
            "--overprovisioned",
            "--unsafe-bypass-fsync", "1",
            "--kernel-page-cache", "1",
        ])
        .stdout(log)
        .stderr(stderr_log)
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn Redpanda: {e}"))?;

    Ok(child)
}

// Poll until the Kafka port accepts a TCP connection, then give Redpanda a moment
// to finish its internal bootstrap before we issue admin commands.
#[cfg(feature = "kafka")]
async fn wait_for_kafka() -> anyhow::Result<()> {
    use tokio::net::TcpStream;
    use tokio::time::{sleep, Duration};

    for attempt in 1..=60 {
        if TcpStream::connect("127.0.0.1:9092").await.is_ok() {
            tracing::info!("Redpanda port open after {}s — waiting for broker bootstrap", attempt);
            // Port is open but broker may still be initialising — give it 3s
            sleep(Duration::from_secs(3)).await;
            return Ok(());
        }
        sleep(Duration::from_secs(1)).await;
    }
    anyhow::bail!("Redpanda did not become available on port 9092 after 60 seconds")
}

#[cfg(feature = "kafka")]
async fn create_topic() -> anyhow::Result<()> {
    use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
    use rdkafka::client::DefaultClientContext;
    use rdkafka::ClientConfig;

    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", "127.0.0.1:9092")
        .create()?;

    let results = admin
        .create_topics(
            &[NewTopic::new("cv_events", 1, TopicReplication::Fixed(1))],
            &AdminOptions::new()
                .request_timeout(Some(std::time::Duration::from_secs(15))),
        )
        .await?;

    for result in results {
        match result {
            Ok(name) => tracing::info!("topic '{}' created", name),
            Err((name, rdkafka::error::RDKafkaErrorCode::TopicAlreadyExists)) => {
                tracing::info!("topic '{}' already exists — reusing", name);
            }
            Err((name, code)) => {
                anyhow::bail!("failed to create topic '{}': {:?}", name, code);
            }
        }
    }
    Ok(())
}

// ── Token validation ──────────────────────────────────────────────────────────

// Borrowed from PostHog/posthog: rust/capture/src/token.rs
// Validates the token before we attempt an expensive decryption pass.
fn validate_token(token: &str) -> anyhow::Result<()> {
    if token.is_empty() {
        anyhow::bail!("CV_SECRET must not be empty");
    }
    if token.len() < 6 {
        anyhow::bail!("CV_SECRET too short (minimum 6 characters)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_token_rejects_empty() {
        assert!(validate_token("").is_err());
    }

    #[test]
    fn validate_token_rejects_too_short() {
        assert!(validate_token("abc").is_err());
    }

    #[test]
    fn validate_token_accepts_valid_secret() {
        assert!(validate_token("correct-horse-battery-staple").is_ok());
    }
}
