// producer — reads raw_history.json + Firebase, emits PostHog-format events
//
// Sequence:
//   1. Decrypt raw_history.json in memory using CV_SECRET
//   2. Deserialise into RawHistory (tolerant types, all fields Option)
//   3. Normalise: mixed date formats, inconsistent keys, string vs array responsibilities
//   4. Optionally fetch cv/skills from Firebase Firestore REST API (Phase 6)
//   5. Return Vec<CvEvent> — caller is responsible for routing to consumer
//
// All date parsing lives here and only here. Dates leave this module as
// NaiveDate formatted to "%Y-%m-%d" strings. No chrono anywhere else.
//
// Key derivation: PBKDF2-HMAC-SHA256, same parameters as src/bin/encrypt.rs.

use anyhow::{anyhow, Result};
use chrono::NaiveDate;

use cv_engine::crypto;
use crate::models::{CvEvent, RawHistory, RawWorkEntry};

pub fn decrypt(encrypted: &[u8], secret: &str) -> Result<Vec<u8>> {
    crypto::decrypt(encrypted, secret)
        .map_err(|_| anyhow!("Decryption failed — check CV_SECRET is correct"))
}

pub async fn produce(decrypted_json: &[u8]) -> Result<Vec<CvEvent>> {
    let raw: RawHistory = serde_json::from_slice(decrypted_json)
        .map_err(|e| anyhow!("Failed to parse raw_history.json: {e}"))?;

    let mut events: Vec<CvEvent> = Vec::new();

    events.push(make_profile_event(&raw)?);

    for (i, entry) in raw.work_history.iter().enumerate() {
        events.extend(make_work_events(entry, i as i32 + 1)?);
    }

    for (i, entry) in raw.education.iter().enumerate() {
        if let Some(event) = make_education_event(entry, i as i32 + 1)? {
            events.push(event);
        }
    }

    // Firebase skills — gracefully skipped until Phase 6
    match fetch_firebase_skills(&raw.firebase_credentials).await {
        Ok(skill_events) => {
            tracing::info!("firebase: {} skill events", skill_events.len());
            events.extend(skill_events);
        }
        Err(e) => tracing::warn!("firebase skills skipped: {e}"),
    }

    Ok(events)
}

// ── Profile ───────────────────────────────────────────────────────────────────

fn make_profile_event(raw: &RawHistory) -> Result<CvEvent> {
    let p = &raw.profile;
    let contact = p.contact.as_ref();

    let affiliations_source = p
        .affiliations
        .as_deref()
        .or(p.outside_work.as_deref())
        .unwrap_or("");
    let interests: Vec<String> = affiliations_source
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.contains('.'))
        .collect();

    Ok(CvEvent::new(
        "profile_update",
        serde_json::json!({
            "name":     p.full_name.as_deref().unwrap_or(""),
            "title":    p.job_title.as_deref().unwrap_or(""),
            "location": format!(
                "{}, {}",
                p.location_city.as_deref().unwrap_or(""),
                p.location_county.as_deref().unwrap_or("")
            ),
            "email":    contact.and_then(|c| c.email_address.as_deref()).unwrap_or(""),
            "linkedin": contact.and_then(|c| c.linkedin_url.as_deref()).unwrap_or(""),
            "github":   contact.and_then(|c| c.github.as_deref()).unwrap_or(""),
            "summary":      p.bio.as_deref().unwrap_or(""),
            "about_me":     p.about_me.as_deref().unwrap_or(""),
            "why_posthog":  p.why_posthog.as_deref().unwrap_or(""),
            "outside_work": p.outside_work.as_deref().unwrap_or(""),
            "interests":    interests,
        }),
    ))
}

// ── Work history ──────────────────────────────────────────────────────────────

fn make_work_events(entry: &RawWorkEntry, sort_order: i32) -> Result<Vec<CvEvent>> {
    let mut events = Vec::new();

    let company = entry.company.as_deref().unwrap_or("Various");
    let title = entry.role.as_deref().unwrap_or("Unknown");
    let id = format!("job_{:03}", sort_order);

    let (start_date, end_date) = extract_dates(entry)?;

    events.push(CvEvent::new(
        "career_milestone",
        serde_json::json!({
            "id":         id,
            "company":    company,
            "title":      title,
            "start_date": start_date.format("%Y-%m-%d").to_string(),
            "end_date":   end_date.map(|d| d.format("%Y-%m-%d").to_string()),
            "sort_order": sort_order,
            "source":     "json",
        }),
    ));

    let responsibilities = normalise_responsibilities(entry.description.as_ref());
    for (i, resp) in responsibilities.iter().enumerate() {
        events.push(CvEvent::new(
            "responsibility_added",
            serde_json::json!({
                "id":              format!("{id}_resp_{:03}", i + 1),
                "career_event_id": id,
                "description":     resp,
                "sort_order":      i as i32 + 1,
            }),
        ));
    }

    // Key projects — inconsistent key names in source data (see data_models.md §6)
    let projects: Vec<String> = entry
        .projects
        .as_deref()
        .or(entry.key_deliverables.as_deref())
        .map(|v| v.to_vec())
        .or_else(|| {
            entry
                .achievements
                .as_deref()
                .map(|s| s.split('\n').map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
        })
        .unwrap_or_default();

    for (i, proj) in projects.iter().enumerate() {
        events.push(CvEvent::new(
            "project_highlight",
            serde_json::json!({
                "id":              format!("{id}_proj_{:03}", i + 1),
                "career_event_id": id,
                "description":     proj,
                "sort_order":      i as i32 + 1,
            }),
        ));
    }

    Ok(events)
}

fn extract_dates(entry: &RawWorkEntry) -> Result<(NaiveDate, Option<NaiveDate>)> {
    // Try individual start/end fields first, then period struct, then "X to Y" string
    let start_str = entry
        .start
        .as_deref()
        .or_else(|| entry.period.as_ref()?.from.as_deref())
        .or_else(|| entry.dates.as_deref()?.split(" to ").next());

    let end_str = entry
        .end
        .as_deref()
        .or_else(|| entry.period.as_ref()?.to.as_deref())
        .or_else(|| entry.dates.as_deref()?.split(" to ").nth(1));

    let start = start_str
        .map(parse_date_flexible)
        .transpose()?
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());

    let end = end_str
        .filter(|s| !s.eq_ignore_ascii_case("present"))
        .map(parse_date_flexible)
        .transpose()?;

    Ok((start, end))
}

// Single date parsing point for the entire project.
// Handles the formats that appear in raw_history.json (see data_models.md §6):
//   "October 2023"  — full month name + 4-digit year
//   "Sep-22"        — abbreviated month + 2-digit year (explicit pivot: ≤68→20xx, ≥69→19xx)
//   "Jul 2017"      — abbreviated month + 4-digit year
//   "2020-10"       — ISO year-month
//   "2020-10-01"    — full ISO date (used in "dates" range strings)
//
// Note: chrono's %y doesn't reliably apply century pivoting in parse context, so
// we handle "Mmm-YY" format manually to avoid year-22-as-AD-0022 misparse.
fn parse_date_flexible(s: &str) -> Result<NaiveDate> {
    let s = s.trim();

    // Full ISO date — most specific, try first
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(d);
    }

    // "2020-10" — ISO year-month only (must check before the month-name paths)
    if let Ok(d) = NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d") {
        return Ok(d);
    }

    // All remaining formats have a month name followed by a separator and a year.
    // We normalise the input manually to avoid chrono's flexible whitespace quirk
    // where `%B %Y` and `%b %Y` consume the separator as zero whitespace then read
    // a signed year (e.g. "Sep-22" → %Y reads "-22" → year = -22 = 23 BC).
    //
    // Strategy: split on the separator, normalise the year to 4 digits, then parse
    // with a safe "%b %Y %d" format using a space-separated reconstructed string.
    let sep = if s.contains('-') { '-' } else { ' ' };
    let parts: Vec<&str> = s.splitn(2, sep).collect();

    if parts.len() == 2 {
        let month_part = parts[0].trim();
        let year_part = parts[1].trim();

        // Determine 4-digit year from either 2-digit or 4-digit year string
        let year_4: Option<i32> = if year_part.len() == 2 {
            year_part.parse::<i32>().ok().map(|yy| {
                // C99 pivot: 00-68 → 2000-2068, 69-99 → 1969-1999
                if yy <= 68 { 2000 + yy } else { 1900 + yy }
            })
        } else if year_part.len() == 4 {
            year_part.parse::<i32>().ok()
        } else {
            None
        };

        if let Some(year) = year_4 {
            let reformatted = format!("{} {} 01", month_part, year);
            if let Ok(d) = NaiveDate::parse_from_str(&reformatted, "%b %Y %d") {
                return Ok(d);
            }
            // Full month name ("October 2023")
            if let Ok(d) = NaiveDate::parse_from_str(&reformatted, "%B %Y %d") {
                return Ok(d);
            }
        }
    }

    Err(anyhow!("unrecognised date format: '{s}'"))
}

// Responsibilities field can be a \n-joined string (Wealth Wizards, Envelop Risk achievements)
// or an array of strings (Heni duties, Polecat, Hargreaves Lansdown).
fn normalise_responsibilities(val: Option<&serde_json::Value>) -> Vec<String> {
    match val {
        None => vec![],
        Some(serde_json::Value::String(s)) => s
            .split('\n')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::to_string)
            .collect(),
        _ => vec![],
    }
}

// ── Education ─────────────────────────────────────────────────────────────────

fn make_education_event(
    entry: &crate::models::RawEducationEntry,
    sort_order: i32,
) -> Result<Option<CvEvent>> {
    let qualification = match entry.qualification.as_deref() {
        Some(q) => q,
        None => return Ok(None),
    };

    let year: i32 = match &entry.year {
        Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(0) as i32,
        Some(serde_json::Value::String(s)) => s.trim().parse().unwrap_or(0),
        _ => return Ok(None),
    };

    if year == 0 {
        return Ok(None);
    }

    let entry_type = classify_education(qualification, entry.institution.as_deref());

    Ok(Some(CvEvent::new(
        "education_entry",
        serde_json::json!({
            "id":            format!("edu_{:03}", sort_order),
            "year":          year,
            "qualification": qualification,
            "institution":   entry.institution.as_deref(),
            "type":          entry_type,
            "sort_order":    sort_order,
            "source":        "json",
        }),
    )))
}

fn classify_education(qualification: &str, institution: Option<&str>) -> &'static str {
    let q = qualification.to_lowercase();
    if q.starts_with("bsc") || q.contains("bachelor") || q.contains("master") {
        return "degree";
    }
    if q.starts_with("diploma") {
        return "diploma";
    }
    if institution.is_some_and(|i| {
        let i = i.to_lowercase();
        i.contains("university") || i.contains("college")
    }) {
        return "degree";
    }
    "certification"
}

// ── Redpanda emit (Phase 5) ───────────────────────────────────────────────────
//
// Serialises each CvEvent as JSON and sends it to the cv_events topic.
// A "pipeline_done" sentinel is sent last so the consumer knows when to stop.
// Only compiled when the kafka feature is enabled (Linux/Docker build).

#[cfg(feature = "kafka")]
pub async fn emit(events: &[CvEvent]) -> Result<()> {
    use rdkafka::producer::{FutureProducer, FutureRecord};
    use rdkafka::ClientConfig;
    use std::time::Duration;

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "127.0.0.1:9092")
        .set("message.timeout.ms", "10000")
        .create()?;

    for event in events {
        let payload = serde_json::to_string(event)?;
        producer
            .send(
                FutureRecord::to("cv_events")
                    .payload(&payload)
                    .key(event.event.as_str()),
                Duration::from_secs(10),
            )
            .await
            .map_err(|(e, _)| anyhow!("Redpanda delivery failed: {e}"))?;
    }

    // Sentinel — tells consumer to stop reading
    let done = serde_json::json!({
        "event": "pipeline_done",
        "distinct_id": "",
        "token": "",
        "timestamp": chrono::Utc::now(),
        "properties": {}
    });
    let done_payload = serde_json::to_string(&done)?;
    producer
        .send(
            FutureRecord::to("cv_events")
                .payload(&done_payload)
                .key("pipeline_done"),
            Duration::from_secs(10),
        )
        .await
        .map_err(|(e, _)| anyhow!("Redpanda sentinel delivery failed: {e}"))?;

    tracing::info!("emitted {} events + pipeline_done to cv_events", events.len());
    Ok(())
}

// ── Firebase (Phase 6) ────────────────────────────────────────────────────────

async fn fetch_firebase_skills(credentials: &serde_json::Value) -> Result<Vec<CvEvent>> {
    let private_key = credentials["private_key"].as_str().unwrap_or("");
    if private_key.is_empty() || private_key.starts_with("REPLACE") {
        anyhow::bail!("placeholder credentials — configure Firebase service account in raw_history.json");
    }

    let token = bearer_token(credentials).await?;
    let project = credentials["project_id"].as_str().unwrap_or("battleplan-dev-2024");

    let skills_doc = fetch_firestore_doc(&token, project, "skills").await?;
    let highlights_doc = fetch_firestore_doc(&token, project, "highlights").await?;

    let mut events: Vec<CvEvent> = Vec::new();

    // cv/skills → { items: [ { skill, category, level, sort_order }, ... ] }
    if let Some(items) = skills_doc["fields"]["items"]["arrayValue"]["values"].as_array() {
        tracing::info!("firebase: {} skills", items.len());
        for (i, item) in items.iter().enumerate() {
            let f = &item["mapValue"]["fields"];
            events.push(CvEvent::new(
                "skill_added",
                serde_json::json!({
                    "id":         format!("skill_{:03}", i + 1),
                    "skill":      fs_str(f, "skill"),
                    "category":   fs_str(f, "category"),
                    "level":      fs_str_opt(f, "level"),
                    "sort_order": fs_int(f, "sort_order").unwrap_or(i as i64 + 1),
                    "source":     "firebase",
                }),
            ));
        }
    }

    // cv/highlights → { items: [ { career_event_id, description, sort_order }, ... ] }
    if let Some(items) = highlights_doc["fields"]["items"]["arrayValue"]["values"].as_array() {
        tracing::info!("firebase: {} highlights", items.len());
        for (i, item) in items.iter().enumerate() {
            let f = &item["mapValue"]["fields"];
            events.push(CvEvent::new(
                "project_highlight",
                serde_json::json!({
                    "id":              format!("highlight_{:03}", i + 1),
                    "career_event_id": fs_str(f, "career_event_id"),
                    "description":     fs_str(f, "description"),
                    "sort_order":      fs_int(f, "sort_order").unwrap_or(i as i64 + 1),
                }),
            ));
        }
    }

    Ok(events)
}

/// Exchange the service account JSON credentials for a short-lived OAuth2 bearer token.
/// Shared with api.rs for the live availability fetch.
pub(crate) async fn bearer_token(credentials: &serde_json::Value) -> Result<String> {
    use gcp_auth::TokenProvider as _;
    let json = serde_json::to_string(credentials)?;
    let sa = gcp_auth::CustomServiceAccount::from_json(&json)
        .map_err(|e| anyhow::anyhow!("invalid service account JSON: {e}"))?;
    let scopes = &["https://www.googleapis.com/auth/datastore"];
    let token = sa.token(scopes).await
        .map_err(|e| anyhow::anyhow!("failed to obtain GCP token: {e}"))?;
    Ok(token.as_str().to_string())
}

async fn fetch_firestore_doc(token: &str, project: &str, doc_id: &str) -> Result<serde_json::Value> {
    let url = format!(
        "https://firestore.googleapis.com/v1/projects/{project}/databases/(default)/documents/cv/{doc_id}"
    );
    let doc = reqwest::Client::new()
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    Ok(doc)
}

fn fs_str<'a>(fields: &'a serde_json::Value, key: &str) -> &'a str {
    fields[key]["stringValue"].as_str().unwrap_or("")
}

fn fs_str_opt(fields: &serde_json::Value, key: &str) -> Option<String> {
    fields[key]["stringValue"].as_str().map(str::to_string)
}

fn fs_int(fields: &serde_json::Value, key: &str) -> Option<i64> {
    fields[key]["integerValue"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| fields[key]["integerValue"].as_i64())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_parsing_handles_all_formats() {
        let cases = vec![
            ("October 2023", "2023-10-01"),
            ("Sep-22",       "2022-09-01"),
            ("May-23",       "2023-05-01"),
            ("Oct-18",       "2018-10-01"),
            ("Jun-20",       "2020-06-01"),
            ("Jul 2017",     "2017-07-01"),
            ("Oct 2018",     "2018-10-01"),
            ("2020-10",      "2020-10-01"),
            ("2020-10-01",   "2020-10-01"),
        ];
        for (input, expected) in cases {
            let result = parse_date_flexible(input).unwrap();
            assert_eq!(result.to_string(), expected, "failed for input: '{}'", input);
        }
    }
}

