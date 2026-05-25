// api — Axum router and all HTTP handlers
//
// Routes:
//   GET /api/profile       → dim_profile + live availability from Firebase
//   GET /api/profile/photo → PNG decrypted at startup (not in static/)
//   GET /api/experience → fact_career_events + dim_responsibilities + dim_key_projects
//   GET /api/skills     → dim_tech_stack grouped by category
//   GET /api/education  → dim_education
//   GET /api/pipeline   → fact_pipeline_run + fact_pipeline_event_log
//   GET /*              → pre-built Vite static files from ./static

use crate::{db, producer};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use duckdb::Connection;
use std::sync::{Arc, Mutex};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
struct AppState {
    conn: Arc<Mutex<Connection>>,
    firebase_credentials: serde_json::Value,
    photo_png: Arc<[u8]>,
    startup_log: Arc<[String]>,
}

pub fn router(
    conn: Arc<Mutex<Connection>>,
    firebase_credentials: serde_json::Value,
    photo_png: Vec<u8>,
    startup_log: Vec<String>,
) -> Router {
    let state = AppState {
        conn,
        firebase_credentials,
        photo_png: Arc::from(photo_png.into_boxed_slice()),
        startup_log: Arc::from(startup_log.into_boxed_slice()),
    };
    Router::new()
        .route("/api/profile", get(get_profile))
        .route("/api/profile/photo", get(get_profile_photo))
        .route("/api/experience", get(get_experience))
        .route("/api/skills", get(get_skills))
        .route("/api/education", get(get_education))
        .route("/api/pipeline", get(get_pipeline))
        .route("/api/pipeline/logs", get(get_pipeline_logs))
        // Serve pre-built Vite output from ./static.
        // Falls back to index.html so React Router deep links work.
        .fallback_service(
            ServeDir::new("static")
                .not_found_service(ServeFile::new("static/index.html")),
        )
        .with_state(state)
}

// ── Handlers ─────────────────────────────────────────────────────────────────

async fn get_profile(State(state): State<AppState>) -> Response {
    let result = { db::query_profile(&state.conn.lock().unwrap()) };
    let mut data = match result {
        Ok(d) => d,
        Err(e) => return db_error("query_profile", e),
    };

    let availability = fetch_availability(&state.firebase_credentials).await;
    if let serde_json::Value::Object(ref mut obj) = data {
        obj.insert("availability".to_string(), availability);
    }

    Json(data).into_response()
}

async fn get_profile_photo(State(state): State<AppState>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("image/png"),
        )
        .header(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))
        .body(Body::from(state.photo_png.as_ref().to_vec()))
        .unwrap()
        .into_response()
}

async fn get_experience(State(state): State<AppState>) -> Response {
    let result = { db::query_experience(&state.conn.lock().unwrap()) };
    match result {
        Ok(data) => Json(data).into_response(),
        Err(e) => db_error("query_experience", e),
    }
}

async fn get_skills(State(state): State<AppState>) -> Response {
    let result = { db::query_skills(&state.conn.lock().unwrap()) };
    match result {
        Ok(data) => Json(data).into_response(),
        Err(e) => db_error("query_skills", e),
    }
}

async fn get_education(State(state): State<AppState>) -> Response {
    let result = { db::query_education(&state.conn.lock().unwrap()) };
    match result {
        Ok(data) => Json(data).into_response(),
        Err(e) => db_error("query_education", e),
    }
}

async fn get_pipeline(State(state): State<AppState>) -> Response {
    let result = { db::query_pipeline(&state.conn.lock().unwrap()) };
    match result {
        Ok(data) => Json(data).into_response(),
        Err(e) => db_error("query_pipeline", e),
    }
}

async fn get_pipeline_logs(State(state): State<AppState>) -> Response {
    Json(state.startup_log.as_ref()).into_response()
}

// ── Firebase availability (live fetch on every /api/profile request) ──────────

async fn fetch_availability(credentials: &serde_json::Value) -> serde_json::Value {
    let private_key = credentials["private_key"].as_str().unwrap_or("");
    if private_key.is_empty() || private_key.starts_with("REPLACE") {
        tracing::warn!("firebase availability: no credentials");
        return serde_json::Value::Null;
    }

    match fetch_availability_inner(credentials).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("firebase availability fetch failed: {e}");
            serde_json::Value::Null
        }
    }
}

async fn fetch_availability_inner(
    credentials: &serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let token = producer::bearer_token(credentials).await?;
    let project = credentials["project_id"].as_str().unwrap_or("battleplan-dev-2024");
    let url = format!(
        "https://firestore.googleapis.com/v1/projects/{project}/databases/(default)/documents/cv/availability"
    );
    let doc: serde_json::Value = reqwest::Client::new()
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let fields = &doc["fields"];
    Ok(serde_json::json!({
        "available":  fields["available"]["booleanValue"].as_bool().unwrap_or(false),
        "from_date":  fields["from_date"]["stringValue"].as_str().unwrap_or(""),
        "message":    fields["message"]["stringValue"].as_str().unwrap_or(""),
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn db_error(context: &str, err: anyhow::Error) -> Response {
    tracing::error!("{context} failed: {err}");
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}
