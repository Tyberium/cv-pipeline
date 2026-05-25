# Data Models: CV Pipeline

Working backwards from the frontend UI to the DuckDB schema,
then upstream to the Redpanda event catalog and source data.

**Authoring:** prose drafts in `planning/copy.md` → `raw_history_plain.json` → `CV_SECRET=... cargo run --bin encrypt` (see `README.md`).

---

## 1. Frontend Pages (Complete List)

The plan listed 3 pages. After mapping against `copy.md`, the full list is:

| Page | Route | Primary API | Notes |
|---|---|---|---|
| About / Hero | `/` | `/api/profile`, `/api/profile/photo` | Hero + About Me + Why PostHog; photo from encrypted `photo.enc` |
| Experience | `/experience` | `/api/experience` | Jobs + responsibilities + key projects |
| Skills | `/skills` | `/api/skills` | Grouped by category, with level badges |
| Education | `/education` | `/api/education` | Certs + degrees, reverse chron |
| Pipeline Health | `/pipeline` | `/api/pipeline` | Meta-page: the pipeline about itself |

The current plan was missing **About** and **Skills** as named pages.
All five pages should be in the frontend `pages/` directory.

---

## 2. API Response Shapes

These are the JSON contracts between Axum and the React frontend.
The DuckDB schema is reverse-engineered from these shapes.

### `GET /api/profile`

```json
{
  "name": "Dave Carroll",
  "title": "Data Engineer",
  "location": "Portishead, North Somerset",
  "email": "d.carroll@gmx.com",
  "linkedin": "https://linkedin.com/in/dave-carroll-9b8b06139",
  "github": "https://github.com/Tyberium",
  "summary": "I am an accomplished data engineer with over 5 years experience...",
  "availability": {
    "available": true,
    "message": "Open to new roles from July 2026"
  },
  "interests": ["Kiting", "Southwest Amateur Gamers", "Reading", "DIY", "PC/PS5 Gaming", "Warhammer", "Dog Training"]
}
```

`availability` comes from Firebase (dynamic). Everything else from `dim_profile` (DuckDB).

### `GET /api/profile/photo`

Returns `image/png` bytes. Not stored in DuckDB or Vite `static/`.

| Artifact | In repo | In image | Plaintext |
|----------|---------|----------|-----------|
| `photo.png` | No (gitignored, local only) | No | Dev source |
| `photo.enc` | Yes | Yes | AES-256-GCM with same `CV_SECRET` as `raw_history.json` |

Decrypted once at container startup; held in memory for the process lifetime. Wrong `CV_SECRET` → container never reaches Axum (same gate as CV JSON).

---

### `GET /api/experience`

```json
[
  {
    "id": "ww_001",
    "company": "Wealth Wizards",
    "title": "Data Engineer",
    "start_date": "2023-10-01",
    "end_date": null,
    "responsibilities": [
      "Lead operational excellence: Oversee daily maintenance...",
      "Drive architectural efficiency: Optimise, simplify..."
    ],
    "key_projects": [
      {
        "id": "ww_proj_001",
        "description": "Designed and deployed critical data pipelines with zero downtime..."
      }
    ],
    "source": "json"
  }
]
```

Ordered by `sort_order` ASC (most recent = lowest number = first).

---

### `GET /api/skills`

```json
{
  "cloud": [
    { "skill": "AWS", "level": "expert" },
    { "skill": "GCP", "level": "proficient" }
  ],
  "languages": [
    { "skill": "Python", "level": "expert" },
    { "skill": "SQL", "level": "expert" },
    { "skill": "Rust", "level": "proficient" }
  ],
  "data": [
    { "skill": "Databricks", "level": "expert" },
    { "skill": "Spark / PySpark", "level": "expert" },
    { "skill": "Kafka", "level": "proficient" },
    { "skill": "DuckDB", "level": "proficient" }
  ],
  "devops": [
    { "skill": "Terraform", "level": "proficient" },
    { "skill": "GitHub Actions", "level": "expert" },
    { "skill": "Docker", "level": "proficient" }
  ],
  "databases": [
    { "skill": "PostgreSQL", "level": "proficient" },
    { "skill": "Elasticsearch / Solr", "level": "familiar" }
  ],
  "other": [
    { "skill": "Linux", "level": "proficient" },
    { "skill": "Git", "level": "expert" }
  ]
}
```

Grouped by `category`. Skill levels and categories come from Firebase (so they can be tuned
without a rebuild — e.g. bumping Rust from `familiar` to `proficient` as it improves).

---

### `GET /api/education`

```json
[
  {
    "id": "edu_deg_001",
    "year": 2007,
    "qualification": "Diploma in IT",
    "institution": "Liverpool John Moores University",
    "type": "degree",
    "sort_order": 1
  },
  {
    "id": "edu_cert_001",
    "year": 2022,
    "qualification": "Data Engineering with Databricks",
    "institution": null,
    "type": "certification",
    "sort_order": 2
  }
]
```

`institution` is nullable — most of the copy's entries are vendor certs with no named institution.
`type` drives frontend rendering (degree badge vs cert badge).

---

### `GET /api/pipeline`

```json
{
  "run_id": "run_20260522_082300",
  "events_produced": 48,
  "events_consumed": 48,
  "duration_ms": 2340,
  "ran_at": "2026-05-22T08:23:00Z",
  "source_breakdown": {
    "json": 21,
    "firebase": 27
  },
  "event_breakdown": {
    "profile_update": 1,
    "career_milestone": 8,
    "responsibility_added": 16,
    "project_highlight": 9,
    "skill_added": 22,
    "education_entry": 20
  }
}
```

Pulled from `fact_pipeline_run`. The Pipeline Health page renders this live.

---

---

## 3. DuckDB Schema (Complete)

```sql
-- ────────────────────────────────────────────────────────────
-- PROFILE
-- Single-row table. id is always 1.
-- ────────────────────────────────────────────────────────────
CREATE TABLE dim_profile (
    id          INTEGER PRIMARY KEY,   -- always 1
    name        VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    location    VARCHAR,
    email       VARCHAR,
    linkedin    VARCHAR,
    github      VARCHAR,
    summary     VARCHAR,
    interests   VARCHAR[]              -- DuckDB native list type
);

-- ────────────────────────────────────────────────────────────
-- EXPERIENCE
-- ────────────────────────────────────────────────────────────
CREATE TABLE fact_career_events (
    id          VARCHAR PRIMARY KEY,   -- e.g. 'ww_001', 'heni_001'
    company     VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    start_date  DATE    NOT NULL,
    end_date    DATE,                  -- NULL = current role
    sort_order  INTEGER NOT NULL,      -- 1 = most recent
    source      VARCHAR NOT NULL       -- 'json' | 'firebase'
);

CREATE TABLE dim_responsibilities (
    id              VARCHAR PRIMARY KEY,
    career_event_id VARCHAR NOT NULL REFERENCES fact_career_events(id),
    description     VARCHAR NOT NULL,
    sort_order      INTEGER NOT NULL
);

CREATE TABLE dim_key_projects (
    id              VARCHAR PRIMARY KEY,
    career_event_id VARCHAR NOT NULL REFERENCES fact_career_events(id),
    description     VARCHAR NOT NULL,
    sort_order      INTEGER NOT NULL
);

-- ────────────────────────────────────────────────────────────
-- SKILLS
-- ────────────────────────────────────────────────────────────
CREATE TABLE dim_tech_stack (
    id          VARCHAR PRIMARY KEY,
    skill       VARCHAR NOT NULL,
    category    VARCHAR NOT NULL,      -- 'cloud' | 'languages' | 'data' | 'devops' | 'databases' | 'other'
    level       VARCHAR,               -- 'expert' | 'proficient' | 'familiar' | NULL
    sort_order  INTEGER NOT NULL,
    source      VARCHAR NOT NULL       -- 'json' | 'firebase'
);

-- ────────────────────────────────────────────────────────────
-- EDUCATION
-- ────────────────────────────────────────────────────────────
CREATE TABLE dim_education (
    id              VARCHAR PRIMARY KEY,
    year            INTEGER NOT NULL,
    qualification   VARCHAR NOT NULL,
    institution     VARCHAR,           -- NULL for vendor certs / online courses
    type            VARCHAR NOT NULL,  -- 'degree' | 'diploma' | 'certification' | 'course'
    sort_order      INTEGER NOT NULL,
    source          VARCHAR NOT NULL   -- 'json' | 'firebase'
);

-- ────────────────────────────────────────────────────────────
-- PIPELINE TELEMETRY
-- ────────────────────────────────────────────────────────────
CREATE TABLE fact_pipeline_run (
    run_id              VARCHAR PRIMARY KEY,
    events_produced     INTEGER NOT NULL,
    events_consumed     INTEGER NOT NULL,
    duration_ms         INTEGER NOT NULL,
    ran_at              TIMESTAMP NOT NULL,
    json_events         INTEGER NOT NULL,
    firebase_events     INTEGER NOT NULL
);

CREATE TABLE fact_pipeline_event_log (
    id              VARCHAR PRIMARY KEY,
    run_id          VARCHAR NOT NULL REFERENCES fact_pipeline_run(run_id),
    event_type      VARCHAR NOT NULL,
    count           INTEGER NOT NULL
);

```

### Schema Change Log vs `final_plan.md`

| Original | Changed to | Reason |
|---|---|---|
| `fact_career_events.summary` | Removed | Split into `dim_responsibilities` + `dim_key_projects` — the copy has 3-5 bullet points per job |
| `dim_education.institution` | Kept but nullable | Most entries in the copy are certs with no institution |
| *(missing)* | `dim_profile` added | Profile text, contact details, interests need a home |
| *(missing)* | `dim_responsibilities` added | Job bullet points |
| *(missing)* | `dim_key_projects` added | Key project bullet points |
| `dim_tech_stack.skill VARCHAR PRIMARY KEY` | `id VARCHAR PRIMARY KEY` + `skill VARCHAR` | Skills may share names across categories |
| `fact_pipeline_run` | Extended + `fact_pipeline_event_log` added | Pipeline Health page needs event type breakdown |

---

## 4. Redpanda Event Catalog

All events use the PostHog capture event schema. The `event` field tells the consumer
which table to write to.

```
cv_events topic
│
├── profile_update        → dim_profile
├── career_milestone      → fact_career_events
├── responsibility_added  → dim_responsibilities
├── project_highlight     → dim_key_projects
├── skill_added           → dim_tech_stack
└── education_entry       → dim_education
```

### Event Schemas

#### `profile_update`
```json
{
  "event": "profile_update",
  "distinct_id": "dave_carroll",
  "token": "cv_pipeline_v1",
  "timestamp": "2026-05-22T08:23:00Z",
  "properties": {
    "id": 1,
    "name": "Dave Carroll",
    "title": "Data Engineer",
    "location": "Portishead, North Somerset",
    "email": "d.carroll@gmx.com",
    "linkedin": "https://linkedin.com/in/dave-carroll-9b8b06139",
    "github": "https://github.com/Tyberium",
    "summary": "I am an accomplished data engineer...",
    "interests": ["Kiting", "Southwest Amateur Gamers", "Reading", "DIY", "PC/PS5 Gaming", "Warhammer", "Dog Training"]
  }
}
```

#### `career_milestone`
```json
{
  "event": "career_milestone",
  "distinct_id": "dave_carroll",
  "token": "cv_pipeline_v1",
  "timestamp": "2026-05-22T08:23:00Z",
  "properties": {
    "id": "ww_001",
    "company": "Wealth Wizards",
    "title": "Data Engineer",
    "start_date": "2023-10-01",
    "end_date": null,
    "sort_order": 1,
    "source": "json"
  }
}
```

#### `responsibility_added`
```json
{
  "event": "responsibility_added",
  "distinct_id": "dave_carroll",
  "token": "cv_pipeline_v1",
  "timestamp": "2026-05-22T08:23:00Z",
  "properties": {
    "id": "ww_resp_001",
    "career_event_id": "ww_001",
    "description": "Lead operational excellence: Oversee daily maintenance...",
    "sort_order": 1
  }
}
```

#### `project_highlight`
```json
{
  "event": "project_highlight",
  "distinct_id": "dave_carroll",
  "token": "cv_pipeline_v1",
  "timestamp": "2026-05-22T08:23:00Z",
  "properties": {
    "id": "ww_proj_001",
    "career_event_id": "ww_001",
    "description": "Designed and deployed critical data pipelines with zero downtime...",
    "sort_order": 1
  }
}
```

#### `skill_added`
```json
{
  "event": "skill_added",
  "distinct_id": "dave_carroll",
  "token": "cv_pipeline_v1",
  "timestamp": "2026-05-22T08:23:00Z",
  "properties": {
    "id": "skill_aws",
    "skill": "AWS",
    "category": "cloud",
    "level": "expert",
    "sort_order": 1,
    "source": "firebase"
  }
}
```

#### `education_entry`
```json
{
  "event": "education_entry",
  "distinct_id": "dave_carroll",
  "token": "cv_pipeline_v1",
  "timestamp": "2026-05-22T08:23:00Z",
  "properties": {
    "id": "edu_cert_001",
    "year": 2022,
    "qualification": "Data Engineering with Databricks",
    "institution": null,
    "type": "certification",
    "sort_order": 2,
    "source": "json"
  }
}
```

---

## 5. Firebase Structure (`battleplan-dev-2024` / `cv` collection)

Firestore documents in the `cv` collection. These are fetched at runtime by the producer.

```
cv/                              ← collection
├── availability                 ← document
│   ├── available: bool
│   ├── from_date: string        "2026-07-01"
│   └── message: string          "Open to new roles from July 2026"
│
├── skills                       ← document
│   └── items: array
│       └── { skill, category, level, sort_order }
│
└── highlights                   ← document (optional overrides for current role)
    └── items: array
        └── { career_event_id, description, sort_order }
```

**What Firebase owns vs JSON:**
- Firebase owns: `availability`, `skills` (levels + categorisation), `highlights` (current role call-outs)
- JSON owns: everything historical — all jobs, all education, contact details, old projects, interests

The practical effect: when Dave changes jobs or updates his skills, he edits Firestore.
No image rebuild needed. The next `docker run` picks it up.

---

## 6. `raw_history.json` Structure

This is the **encrypted** file baked into the image. Decrypted at runtime using `CV_SECRET`.
It is **intentionally messy** — the `...yikes!` log line earns its place.

### Top-level structure (post-decryption)
```json
{
  "meta": {
    "version": 1,
    "encrypted_at": "2026-05-22"
  },
  "firebase_credentials": {
    "type": "service_account",
    "project_id": "battleplan-dev-2024",
    "private_key_id": "...",
    "private_key": "...",
    "client_email": "cv-reader@battleplan-dev-2024.iam.gserviceaccount.com"
  },
  "profile": {
    "full_name": "Dave Carroll",
    "job_title": "Data Engineer",
    "location_city": "Portishead",
    "location_county": "North Somerset",
    "contact": {
      "email_address": "d.carroll@gmx.com",
      "linkedin_url": "https://linkedin.com/in/dave-carroll-9b8b06139",
      "github": "https://github.com/Tyberium"
    },
    "bio": "I am an accomplished data engineer...",
    "outside_work": "Data Bristol, Kiting, Southwest Amateur Gamers, Reading, DIY, PC/PS5 Gaming, Warhammer, Dog Training"
  },
  "work_history": [
    {
      "employer": "Wealth Wizards",
      "role": "Data Engineer",
      "start": "October 2023",
      "end": "Present",
      "description": "Lead operational excellence: Oversee daily maintenance...\nDrive architectural efficiency: ...",
      "projects": [
        "Designed and deployed critical data pipelines...",
        "Re-architected and migrated a six-year-old multi-tenant Datalake..."
      ]
    },
    {
      "employer": "Heni",
      "job_title": "Data Engineer",
      "period": {
        "from": "Sep-22",
        "to": "May-23"
      },
      "duties": [
        "Develop and deploy efficient batch and streaming pipelines...",
        "Work with data scientists and stakeholders..."
      ],
      "key_deliverables": [
        "Developed pipelines to ingest Blockchain (Ethereum & Palm) transaction data..."
      ]
    },
    {
      "company_name": "Envelop Risk",
      "position": "Data Engineer",
      "dates": "2020-10 to 2022-09",
      "overview": "Defined data platform strategy...",
      "achievements": "Built a cross team consensus..."
    }
  ],
  "education": [
    { "yr": 2022, "cert": "Data Engineering with Databricks" },
    { "yr": 2022, "cert": "Fundamentals of the Databricks Lakehouse Platform" },
    { "year": "2007", "qual": "Diploma in IT", "uni": "Liverpool John Moores University" },
    { "year": "2003", "qualification": "BSc Genetic Engineering", "institution": "The University of Leicester" }
  ]
}
```

### Intentional Messiness (for the `...yikes!` log moment)

| Field | Problem | Producer fix |
|---|---|---|
| `start` / `end` | "October 2023", "Present", "Sep-22", "2020-10" — four date formats | Normalise to `DATE` with a date parser |
| Job title field | `role`, `job_title`, `position` — all different keys across entries | Key aliasing / fallback chain |
| Responsibilities | Single `\n`-joined string in some entries, array in others | Split string on `\n` or use as-is if array |
| Key projects | `projects`, `key_deliverables`, `achievements` — all different keys | Key aliasing |
| `interests` / `outside_work` | Comma-separated string, not an array | Split on `,` and trim |
| Education `year` | Integer in some entries, string in others | Parse to integer |
| Education field names | `cert`, `qual`, `qualification` — three key names | Key aliasing |

The producer's "Bringing order to Chaos, tidying up JSON data" log line covers this normalisation step.

---

## 7. Data Lineage Map

```
raw_history.json (encrypted)
    │
    ▼ CV_SECRET decrypt
    │
    ├── profile.*          ──────────────────► profile_update event
    ├── work_history[]     ──────────────────► career_milestone events
    │   ├── responsibilities/duties           ► responsibility_added events
    │   └── projects/achievements             ► project_highlight events
    └── education[]        ──────────────────► education_entry events (from json)

Firebase cv/skills         ──────────────────► skill_added events
Firebase cv/highlights     ──────────────────► project_highlight events (source=firebase)
Firebase cv/availability   ──────────────────► NOT an event — merged into /api/profile response
                                               (fetched at query time, not pipeline time)

All events → cv_events (Redpanda topic)
    │
    ▼ consumer.rs
    │
    ├── profile_update        → dim_profile
    ├── career_milestone      → fact_career_events
    ├── responsibility_added  → dim_responsibilities
    ├── project_highlight     → dim_key_projects
    ├── skill_added           → dim_tech_stack
    └── education_entry       → dim_education

DuckDB cv_gold.db
    │
    ▼ Axum API
    │
    ├── /api/profile      ← dim_profile + Firebase availability (live join)
    ├── /api/experience   ← fact_career_events + dim_responsibilities + dim_key_projects
    ├── /api/skills       ← dim_tech_stack
    ├── /api/education    ← dim_education
    └── /api/pipeline     ← fact_pipeline_run + fact_pipeline_event_log
```

### Note on `availability`

Firebase `cv/availability` is intentionally **not** sent through Redpanda.
It is fetched fresh at query time by the `/api/profile` handler — this gives
the "current availability" field true dynamism even after the pipeline has run.
The pipeline telemetry (events_produced etc.) should reflect that Firebase was queried
but not treat availability as an event.

---

## 8. `models.rs` Serde Structs (Agent Guidance)

The Rust models layer sits in `src/models.rs`. There are two distinct sets of structs:

**A. Raw input structs** — deserialise the messy JSON and Firebase responses.
These are tolerant (use `Option<>` everywhere, multiple field aliases).

```rust
// Raw work history entry — fields vary per employer, hence all Option
#[derive(Deserialize)]
struct RawWorkEntry {
    // Company name — try multiple keys
    #[serde(alias = "employer", alias = "company_name")]
    company: Option<String>,

    // Title — try multiple keys  
    #[serde(alias = "role", alias = "position", alias = "job_title")]
    title: Option<String>,

    // Date fields — all formats, all optional
    start: Option<String>,
    end: Option<String>,
    #[serde(alias = "period")]
    period: Option<RawPeriod>,
    dates: Option<String>,

    // Responsibilities — string or array
    description: Option<serde_json::Value>,
    #[serde(alias = "duties")]
    responsibilities: Option<serde_json::Value>,

    // Projects — multiple key names
    projects: Option<Vec<String>>,
    key_deliverables: Option<Vec<String>>,
    achievements: Option<String>,
}
```

**B. Clean output structs** — what gets serialised to JSON for API responses.
These match the API response shapes in Section 2 exactly.
They are also used to construct Redpanda events (wrapped in the PostHog envelope).

---

## 9. Resolved Decisions

| # | Decision |
|---|---|
| 1 | `/` is the About/Hero page. Experience, Skills, Education, Pipeline are separate pages reachable via nav. |
| 2 | `company = "Various"` for the Sysadmin, Teacher, and Retail roles — honest and matches the copy. No special handling. |
| 3 | Education sorted reverse-chronological (newest first). `sort_order` in `dim_education` controls this — assign 1 to the most recent entry. |
| 4 | `availability` from Firebase is non-blocking. The pipeline writes to DuckDB and completes. `/api/profile` fetches availability live from Firestore on each request. One HTTP call per page load — acceptable at this scale. |
| 5 | Store `dim_profile.interests` as `VARCHAR` (JSON array string). Axum deserialises it before sending the response. Avoids any uncertainty with `VARCHAR[]` support in the `duckdb` crate. |
