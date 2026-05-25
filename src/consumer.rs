// consumer — reads from cv_events topic, routes by event type, writes to DuckDB
//
// process_event is the core routing function — called once per Redpanda message.
// The rdkafka consumer loop (run()) wires this up in Phase 5.
//
// Unknown event types are logged as warnings — not silent discards, not errors.
// If a new event type appears, it should be added to data_models.md first.

use crate::{db, models::CvEvent};
use anyhow::Result;
use duckdb::Connection;

pub fn process_event(conn: &Connection, event: &CvEvent) -> Result<()> {
    match event.event.as_str() {
        "profile_update"       => db::insert_profile(conn, &event.properties),
        "career_milestone"     => db::insert_career_event(conn, &event.properties),
        "responsibility_added" => db::insert_responsibility(conn, &event.properties),
        "project_highlight"    => db::insert_key_project(conn, &event.properties),
        "skill_added"          => db::insert_skill(conn, &event.properties),
        "education_entry"      => db::insert_education(conn, &event.properties),
        unknown => {
            tracing::warn!("unknown event type: {}", unknown);
            Ok(())
        }
    }
}

// Phase 5: rdkafka consumer loop — reads from cv_events, calls process_event per message.
// Only compiled with the kafka feature (Linux/Docker). On Windows the short-circuit
// path in main.rs is used instead.
#[cfg(feature = "kafka")]
pub async fn run(conn: std::sync::Arc<std::sync::Mutex<duckdb::Connection>>) -> Result<u32> {
    use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
    use rdkafka::message::Message;
    use rdkafka::ClientConfig;

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "127.0.0.1:9092")
        .set("group.id", "cv_consumer")
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "false")
        .create()?;

    consumer.subscribe(&["cv_events"])?;
    tracing::info!("consumer subscribed to cv_events");

    let mut consumed = 0u32;
    loop {
        let msg = consumer.recv().await?;
        let payload = match msg.payload_view::<str>() {
            Some(Ok(s)) => s,
            _ => {
                tracing::warn!("non-UTF8 message on cv_events, skipping");
                continue;
            }
        };

        // Parse event type before full deserialisation — lets us detect the sentinel cheaply
        let value: serde_json::Value = match serde_json::from_str(payload) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("failed to parse message JSON: {e}");
                continue;
            }
        };

        if value["event"].as_str() == Some("pipeline_done") {
            consumer.commit_message(&msg, CommitMode::Async).ok();
            break;
        }

        let event: crate::models::CvEvent = match serde_json::from_value(value) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("failed to deserialise CvEvent: {e}");
                continue;
            }
        };

        {
            let conn_guard = conn.lock().unwrap();
            process_event(&conn_guard, &event)?;
        }
        consumer.commit_message(&msg, CommitMode::Async).ok();
        consumed += 1;
    }

    tracing::info!("consumer processed {} events", consumed);
    Ok(consumed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, models::CvEvent};
    use duckdb::Connection;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn profile_round_trip() {
        let conn = test_conn();

        process_event(&conn, &CvEvent::new("profile_update", serde_json::json!({
            "name": "Dave Carroll",
            "title": "Data Engineer",
            "location": "Portishead, North Somerset",
            "email": "d.carroll@gmx.com",
            "linkedin": "https://linkedin.com/in/dave-carroll-9b8b06139",
            "github": "https://github.com/Tyberium",
            "summary": "Accomplished data engineer with 5+ years experience.",
            "interests": ["Kiting", "Dog Training", "Warhammer"]
        }))).unwrap();

        let profile = db::query_profile(&conn).unwrap();
        assert_eq!(profile["name"], "Dave Carroll");
        assert_eq!(profile["title"], "Data Engineer");
        // Interests survive the JSON string round trip
        assert_eq!(profile["interests"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn experience_round_trip() {
        let conn = test_conn();

        process_event(&conn, &CvEvent::new("career_milestone", serde_json::json!({
            "id": "ww_001", "company": "Wealth Wizards", "title": "Data Engineer",
            "start_date": "2023-10-01", "end_date": null, "sort_order": 1, "source": "json"
        }))).unwrap();

        process_event(&conn, &CvEvent::new("responsibility_added", serde_json::json!({
            "id": "ww_resp_001", "career_event_id": "ww_001",
            "description": "Lead operational excellence.", "sort_order": 1
        }))).unwrap();

        process_event(&conn, &CvEvent::new("project_highlight", serde_json::json!({
            "id": "ww_proj_001", "career_event_id": "ww_001",
            "description": "Reduced AWS bill by 25%.", "sort_order": 1
        }))).unwrap();

        let experience = db::query_experience(&conn).unwrap();
        let jobs = experience.as_array().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0]["company"], "Wealth Wizards");
        assert_eq!(jobs[0]["responsibilities"].as_array().unwrap().len(), 1);
        assert_eq!(jobs[0]["key_projects"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn skills_round_trip() {
        let conn = test_conn();

        process_event(&conn, &CvEvent::new("skill_added", serde_json::json!({
            "id": "skill_aws", "skill": "AWS", "category": "cloud",
            "level": "expert", "sort_order": 1, "source": "firebase"
        }))).unwrap();

        process_event(&conn, &CvEvent::new("skill_added", serde_json::json!({
            "id": "skill_python", "skill": "Python", "category": "languages",
            "level": "expert", "sort_order": 1, "source": "firebase"
        }))).unwrap();

        let skills = db::query_skills(&conn).unwrap();
        assert_eq!(skills["cloud"].as_array().unwrap().len(), 1);
        assert_eq!(skills["cloud"][0]["skill"], "AWS");
        assert_eq!(skills["languages"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn education_round_trip() {
        let conn = test_conn();

        process_event(&conn, &CvEvent::new("education_entry", serde_json::json!({
            "id": "edu_001", "year": 2022,
            "qualification": "Data Engineering with Databricks",
            "institution": null, "type": "certification", "sort_order": 1, "source": "json"
        }))).unwrap();

        process_event(&conn, &CvEvent::new("education_entry", serde_json::json!({
            "id": "edu_002", "year": 2007,
            "qualification": "Diploma in IT",
            "institution": "Liverpool John Moores University",
            "type": "diploma", "sort_order": 2, "source": "json"
        }))).unwrap();

        let education = db::query_education(&conn).unwrap();
        let entries = education.as_array().unwrap();
        assert_eq!(entries.len(), 2);
        // Institution nullable: cert has null, degree has a value
        assert!(entries[0]["institution"].is_null());
        assert_eq!(entries[1]["institution"], "Liverpool John Moores University");
    }

    #[test]
    fn unknown_event_type_does_not_error() {
        let conn = test_conn();
        // Should warn, not panic or return Err
        let result = process_event(&conn, &CvEvent::new("future_event_type", serde_json::json!({})));
        assert!(result.is_ok());
    }
}
