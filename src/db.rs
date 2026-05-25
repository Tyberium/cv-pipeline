// db — DuckDB connection, schema initialisation, all insert and query functions
//
// Rules:
//   - Parameterised queries only — no string interpolation into SQL
//   - Schema lives here; consumer.rs calls init_schema() then insert_* functions
//   - No SQL strings anywhere else in the codebase

use anyhow::Result;
use duckdb::{params, Connection};

pub fn open() -> Result<Connection> {
    let conn = Connection::open("cv_gold.db")?;
    Ok(conn)
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE OR REPLACE TABLE dim_profile (
            id          INTEGER PRIMARY KEY,
            name        VARCHAR NOT NULL,
            title       VARCHAR NOT NULL,
            location    VARCHAR,
            email       VARCHAR,
            linkedin    VARCHAR,
            github      VARCHAR,
            summary     VARCHAR,
            about_me    VARCHAR,
            why_posthog  VARCHAR,
            outside_work VARCHAR,
            interests    VARCHAR  -- JSON array string, parsed in Rust
        );

        CREATE TABLE IF NOT EXISTS fact_career_events (
            id          VARCHAR PRIMARY KEY,
            company     VARCHAR NOT NULL,
            title       VARCHAR NOT NULL,
            start_date  DATE    NOT NULL,
            end_date    DATE,
            sort_order  INTEGER NOT NULL,
            source      VARCHAR NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dim_responsibilities (
            id              VARCHAR PRIMARY KEY,
            career_event_id VARCHAR NOT NULL,
            description     VARCHAR NOT NULL,
            sort_order      INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dim_key_projects (
            id              VARCHAR PRIMARY KEY,
            career_event_id VARCHAR NOT NULL,
            description     VARCHAR NOT NULL,
            sort_order      INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dim_tech_stack (
            id          VARCHAR PRIMARY KEY,
            skill       VARCHAR NOT NULL,
            category    VARCHAR NOT NULL,
            level       VARCHAR,
            sort_order  INTEGER NOT NULL,
            source      VARCHAR NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dim_education (
            id              VARCHAR PRIMARY KEY,
            year            INTEGER NOT NULL,
            qualification   VARCHAR NOT NULL,
            institution     VARCHAR,
            type            VARCHAR NOT NULL,
            sort_order      INTEGER NOT NULL,
            source          VARCHAR NOT NULL
        );

        CREATE TABLE IF NOT EXISTS fact_pipeline_run (
            run_id          VARCHAR PRIMARY KEY,
            events_produced INTEGER NOT NULL,
            events_consumed INTEGER NOT NULL,
            duration_ms     INTEGER NOT NULL,
            ran_at          TIMESTAMP NOT NULL,
            json_events     INTEGER NOT NULL,
            firebase_events INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS fact_pipeline_event_log (
            id          VARCHAR PRIMARY KEY,
            run_id      VARCHAR NOT NULL,
            event_type  VARCHAR NOT NULL,
            count       INTEGER NOT NULL
        );

        ",
    )?;
    Ok(())
}

// ── Insert functions — one per event type ────────────────────────────────────

pub fn insert_profile(conn: &Connection, props: &serde_json::Value) -> Result<()> {
    let interests = serde_json::to_string(&props["interests"])?;
    conn.execute(
        "INSERT OR REPLACE INTO dim_profile
         VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            props["name"].as_str().unwrap_or(""),
            props["title"].as_str().unwrap_or(""),
            props["location"].as_str(),
            props["email"].as_str(),
            props["linkedin"].as_str(),
            props["github"].as_str(),
            props["summary"].as_str(),
            props["about_me"].as_str().unwrap_or(""),
            props["why_posthog"].as_str().unwrap_or(""),
            props["outside_work"].as_str().unwrap_or(""),
            interests,
        ],
    )?;
    Ok(())
}

pub fn insert_career_event(conn: &Connection, props: &serde_json::Value) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO fact_career_events
         VALUES (?, ?, ?, CAST(? AS DATE), CAST(? AS DATE), ?, ?)",
        params![
            props["id"].as_str().unwrap_or(""),
            props["company"].as_str().unwrap_or(""),
            props["title"].as_str().unwrap_or(""),
            props["start_date"].as_str().unwrap_or(""),
            props["end_date"].as_str(),
            props["sort_order"].as_i64().unwrap_or(0) as i32,
            props["source"].as_str().unwrap_or("json"),
        ],
    )?;
    Ok(())
}

pub fn insert_responsibility(conn: &Connection, props: &serde_json::Value) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO dim_responsibilities VALUES (?, ?, ?, ?)",
        params![
            props["id"].as_str().unwrap_or(""),
            props["career_event_id"].as_str().unwrap_or(""),
            props["description"].as_str().unwrap_or(""),
            props["sort_order"].as_i64().unwrap_or(0) as i32,
        ],
    )?;
    Ok(())
}

pub fn insert_key_project(conn: &Connection, props: &serde_json::Value) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO dim_key_projects VALUES (?, ?, ?, ?)",
        params![
            props["id"].as_str().unwrap_or(""),
            props["career_event_id"].as_str().unwrap_or(""),
            props["description"].as_str().unwrap_or(""),
            props["sort_order"].as_i64().unwrap_or(0) as i32,
        ],
    )?;
    Ok(())
}

pub fn insert_skill(conn: &Connection, props: &serde_json::Value) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO dim_tech_stack VALUES (?, ?, ?, ?, ?, ?)",
        params![
            props["id"].as_str().unwrap_or(""),
            props["skill"].as_str().unwrap_or(""),
            props["category"].as_str().unwrap_or("other"),
            props["level"].as_str(),
            props["sort_order"].as_i64().unwrap_or(0) as i32,
            props["source"].as_str().unwrap_or("firebase"),
        ],
    )?;
    Ok(())
}

pub fn insert_education(conn: &Connection, props: &serde_json::Value) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO dim_education VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![
            props["id"].as_str().unwrap_or(""),
            props["year"].as_i64().unwrap_or(0) as i32,
            props["qualification"].as_str().unwrap_or(""),
            props["institution"].as_str(),
            props["type"].as_str().unwrap_or("course"),
            props["sort_order"].as_i64().unwrap_or(0) as i32,
            props["source"].as_str().unwrap_or("json"),
        ],
    )?;
    Ok(())
}

pub fn insert_pipeline_run(conn: &Connection, run: &serde_json::Value) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO fact_pipeline_run VALUES (?, ?, ?, ?, CAST(? AS TIMESTAMP), ?, ?)",
        params![
            run["run_id"].as_str().unwrap_or(""),
            run["events_produced"].as_i64().unwrap_or(0) as i32,
            run["events_consumed"].as_i64().unwrap_or(0) as i32,
            run["duration_ms"].as_i64().unwrap_or(0) as i32,
            run["ran_at"].as_str().unwrap_or(""),
            run["json_events"].as_i64().unwrap_or(0) as i32,
            run["firebase_events"].as_i64().unwrap_or(0) as i32,
        ],
    )?;
    Ok(())
}

pub fn insert_pipeline_event_log(
    conn: &Connection,
    run_id: &str,
    event_type: &str,
    count: i32,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO fact_pipeline_event_log VALUES (?, ?, ?, ?)",
        params![format!("{run_id}_{event_type}"), run_id, event_type, count],
    )?;
    Ok(())
}

// ── Query functions — one per API endpoint ───────────────────────────────────

pub fn query_profile(conn: &Connection) -> Result<serde_json::Value> {
    let row_opt: Option<(
        String, String, Option<String>, Option<String>, Option<String>, Option<String>,
        Option<String>, Option<String>, Option<String>, Option<String>, Option<String>,
    )> = {
        let mut stmt = conn.prepare(
            "SELECT name, title, location, email, linkedin, github, summary, about_me, why_posthog, outside_work, interests
             FROM dim_profile WHERE id = 1",
        )?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            None => None,
            Some(row) => Some((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?,
                row.get(8)?, row.get(9)?, row.get(10)?,
            )),
        }
    };

    match row_opt {
        None => Ok(serde_json::Value::Null),
        Some((name, title, location, email, linkedin, github, summary, about_me, why_posthog, outside_work, interests_raw)) => {
            let interests: Vec<String> = interests_raw
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            Ok(serde_json::json!({
                "name": name,
                "title": title,
                "location": location,
                "email": email,
                "linkedin": linkedin,
                "github": github,
                "summary": summary,
                "about_me": about_me.unwrap_or_default(),
                "why_posthog": why_posthog.unwrap_or_default(),
                "outside_work": outside_work.unwrap_or_default(),
                "availability": null,
                "interests": interests,
            }))
        }
    }
}

pub fn query_experience(conn: &Connection) -> Result<serde_json::Value> {
    // Step 1: collect career events — scoped so the borrow on conn is released
    let job_rows: Vec<(String, String, String, String, Option<String>, i32, String)> = {
        let mut stmt = conn.prepare(
            "SELECT id, company, title,
                    CAST(start_date AS VARCHAR),
                    CAST(end_date AS VARCHAR),
                    sort_order, source
             FROM fact_career_events ORDER BY sort_order",
        )?;
        let mut rows = stmt.query([])?;
        let mut v = Vec::new();
        while let Some(row) = rows.next()? {
            v.push((
                row.get(0)?, row.get(1)?, row.get(2)?,
                row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?,
            ));
        }
        v
    };

    let mut events = Vec::new();

    for (id, company, title, start_date, end_date, sort_order, source) in job_rows {
        // Step 2: responsibilities for this job
        let responsibilities: Vec<String> = {
            let mut stmt = conn.prepare(
                "SELECT description FROM dim_responsibilities
                 WHERE career_event_id = ? ORDER BY sort_order",
            )?;
            let mut rows = stmt.query(params![id])?;
            let mut v = Vec::new();
            while let Some(row) = rows.next()? {
                v.push(row.get(0)?);
            }
            v
        };

        // Step 3: key projects for this job
        let key_projects: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT id, description, sort_order FROM dim_key_projects
                 WHERE career_event_id = ? ORDER BY sort_order",
            )?;
            let mut rows = stmt.query(params![id])?;
            let mut v = Vec::new();
            while let Some(row) = rows.next()? {
                v.push(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "career_event_id": id,
                    "description": row.get::<_, String>(1)?,
                    "sort_order": row.get::<_, i32>(2)?,
                }));
            }
            v
        };

        events.push(serde_json::json!({
            "id": id,
            "company": company,
            "title": title,
            "start_date": start_date,
            "end_date": end_date,
            "sort_order": sort_order,
            "source": source,
            "responsibilities": responsibilities,
            "key_projects": key_projects,
        }));
    }

    Ok(serde_json::Value::Array(events))
}

pub fn query_skills(conn: &Connection) -> Result<serde_json::Value> {
    let mut stmt = conn.prepare(
        "SELECT skill, category, level FROM dim_tech_stack ORDER BY category, sort_order",
    )?;
    let mut rows = stmt.query([])?;

    let mut map: std::collections::BTreeMap<String, Vec<serde_json::Value>> =
        std::collections::BTreeMap::new();

    while let Some(row) = rows.next()? {
        let skill: String = row.get(0)?;
        let category: String = row.get(1)?;
        let level: Option<String> = row.get(2)?;
        map.entry(category).or_default().push(serde_json::json!({
            "skill": skill,
            "level": level,
        }));
    }

    Ok(serde_json::to_value(map)?)
}

pub fn query_education(conn: &Connection) -> Result<serde_json::Value> {
    let mut stmt = conn.prepare(
        "SELECT id, year, qualification, institution, type, sort_order
         FROM dim_education ORDER BY sort_order",
    )?;
    let mut rows = stmt.query([])?;
    let mut entries = Vec::new();

    while let Some(row) = rows.next()? {
        entries.push(serde_json::json!({
            "id":            row.get::<_, String>(0)?,
            "year":          row.get::<_, i32>(1)?,
            "qualification": row.get::<_, String>(2)?,
            "institution":   row.get::<_, Option<String>>(3)?,
            "type":          row.get::<_, String>(4)?,
            "sort_order":    row.get::<_, i32>(5)?,
        }));
    }

    Ok(serde_json::Value::Array(entries))
}

pub fn query_pipeline(conn: &Connection) -> Result<serde_json::Value> {
    // Step 1: most recent run
    let run_opt: Option<(String, i32, i32, i32, String, i32, i32)> = {
        let mut stmt = conn.prepare(
            "SELECT run_id, events_produced, events_consumed, duration_ms,
                    CAST(ran_at AS VARCHAR), json_events, firebase_events
             FROM fact_pipeline_run ORDER BY ran_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            None => None,
            Some(row) => Some((
                row.get(0)?, row.get(1)?, row.get(2)?,
                row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?,
            )),
        }
    };

    let Some((run_id, produced, consumed, duration_ms, ran_at, json_events, firebase_events)) =
        run_opt
    else {
        return Ok(serde_json::Value::Null);
    };

    // Step 2: event type breakdown for this run
    let mut event_breakdown = serde_json::Map::new();
    {
        let mut stmt = conn.prepare(
            "SELECT event_type, count FROM fact_pipeline_event_log WHERE run_id = ?",
        )?;
        let mut rows = stmt.query(params![run_id])?;
        while let Some(row) = rows.next()? {
            let event_type: String = row.get(0)?;
            let count: i32 = row.get(1)?;
            event_breakdown.insert(event_type, serde_json::json!(count));
        }
    }

    Ok(serde_json::json!({
        "run_id":          run_id,
        "events_produced": produced,
        "events_consumed": consumed,
        "duration_ms":     duration_ms,
        "ran_at":          ran_at,
        "source_breakdown": {
            "json":     json_events,
            "firebase": firebase_events,
        },
        "event_breakdown": event_breakdown,
    }))
}

