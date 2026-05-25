// models — all serde structs
#![allow(dead_code)] // Clean* types used from Phase 5 onwards
//
// Two families:
//   Raw*   — tolerant input types for messy raw_history.json (all fields Option,
//             multiple serde aliases to handle inconsistent key names)
//   Clean* — output types that match the API response shapes in planning/data_models.md
//             These are also used as the properties payload inside Redpanda events.
//
// PostHog event envelope — wraps every event emitted to the cv_events topic.
// Borrowed from PostHog/posthog event capture schema — an engineer there should
// recognise this immediately.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ── PostHog event envelope ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CvEvent {
    pub event: String,
    pub distinct_id: String,
    pub token: String,
    pub timestamp: DateTime<Utc>,
    pub properties: serde_json::Value,
}

impl CvEvent {
    pub fn new(event: &str, properties: serde_json::Value) -> Self {
        Self {
            event: event.to_string(),
            distinct_id: "dave_carroll".to_string(),
            token: "cv_pipeline_v1".to_string(),
            timestamp: Utc::now(),
            properties,
        }
    }
}

// ── Raw input types ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RawHistory {
    pub firebase_credentials: serde_json::Value,
    pub profile: RawProfile,
    pub work_history: Vec<RawWorkEntry>,
    pub education: Vec<RawEducationEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RawProfile {
    pub full_name: Option<String>,
    pub job_title: Option<String>,
    pub location_city: Option<String>,
    pub location_county: Option<String>,
    pub contact: Option<RawContact>,
    pub bio: Option<String>,
    pub about_me: Option<String>,
    pub why_posthog: Option<String>,
    pub outside_work: Option<String>,
    /// Comma-separated affiliations for interest pills (e.g. "Data Bristol, Kiting, …").
    pub affiliations: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawContact {
    pub email_address: Option<String>,
    pub linkedin_url: Option<String>,
    pub github: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawWorkEntry {
    // Company — inconsistent key names across entries
    #[serde(alias = "employer", alias = "company_name")]
    pub company: Option<String>,

    // Title — inconsistent key names
    #[serde(alias = "position", alias = "job_title")]
    pub role: Option<String>,

    // Dates — four different formats exist in the source data
    pub start: Option<String>,
    pub end: Option<String>,
    pub period: Option<RawPeriod>,
    pub dates: Option<String>,

    // Responsibilities — string in some entries, array in others
    #[serde(alias = "duties")]
    pub description: Option<serde_json::Value>,

    // Key projects — inconsistent key names
    pub projects: Option<Vec<String>>,
    pub key_deliverables: Option<Vec<String>>,
    pub achievements: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawPeriod {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawEducationEntry {
    // Year — integer in some entries, string in others
    #[serde(alias = "yr")]
    pub year: Option<serde_json::Value>,

    // Qualification — inconsistent key names
    #[serde(alias = "cert", alias = "qual")]
    pub qualification: Option<String>,

    // Institution — not present for most cert entries
    #[serde(alias = "uni")]
    pub institution: Option<String>,
}

// ── Clean output types (match API response shapes) ───────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub title: String,
    pub location: String,
    pub email: String,
    pub linkedin: String,
    pub github: String,
    pub summary: String,
    pub about_me: String,
    pub why_posthog: String,
    pub interests: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CareerEvent {
    pub id: String,
    pub company: String,
    pub title: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub responsibilities: Vec<String>,
    pub key_projects: Vec<KeyProject>,
    pub sort_order: i32,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyProject {
    pub id: String,
    pub career_event_id: String,
    pub description: String,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub skill: String,
    pub category: String,
    pub level: Option<String>,
    pub sort_order: i32,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EducationEntry {
    pub id: String,
    pub year: i32,
    pub qualification: String,
    pub institution: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub sort_order: i32,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineRun {
    pub run_id: String,
    pub events_produced: i32,
    pub events_consumed: i32,
    pub duration_ms: i32,
    pub ran_at: DateTime<Utc>,
    pub json_events: i32,
    pub firebase_events: i32,
}
