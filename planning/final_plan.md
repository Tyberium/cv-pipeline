# Final Plan: CV as a Docker Pipeline

## The Concept

"My CV is a Docker image."

The recruiter runs a single command, watches a humorous pipeline log play out in the terminal,
and ends up with a local website serving the CV.

The pipeline *is* the CV. It mirrors the PostHog architecture and demonstrates the skills
rather than listing them.

---

## The Command

```bash
docker run -p 8080:8080 -e CV_SECRET=<provided-on-request> ghcr.io/tyberium/cv:latest
```

A `docker-compose.yml` will also be provided in the README for convenience.

---

## The Stack (PostHog-Aligned)

| Layer | Technology | PostHog Equivalent |
|---|---|---|
| Ingestion / capture | Rust (`tokio`, `rdkafka`, `serde_json`, `reqwest`) | `capture` service |
| Message bus | Redpanda (Kafka-compatible, single binary) | Kafka |
| Storage | DuckDB (`duckdb` crate) | ClickHouse |
| API + static file server | Axum (Rust web framework) | Django API |
| Frontend | Vite + React + Mantine (pre-built at image build time) | PostHog UI |

**No Python. No Node at runtime. One compiled Rust binary handles everything.**

Node/npm is used at Docker build time to compile the Vite frontend into static files.
Those static files are served by Axum at runtime — npm never runs inside the container.

---

## The Log Script (Terminal UX Narrative)

Plays out during container startup before the web server comes up.

```
Initialising Over-Engineered CV -- Maximum Effort!
Decrypting local json file data using provided secret
Data Quality checks running
...yikes!
Bringing order to Chaos, tidying up JSON data
JSON data contains mapping to true data - connecting to FireBase API
Connection established using SDK, retrieving data within rate limits
Data Quality OK
Data Transform required OLAP to OLTP
Loading enterprise transformation tooling... just kidding, data small, Don't go Big
Walking Dog to think over solution
Coming back from Dog walk, it rained, Chihuahua doesn't do rain
Solution: Rust, Redpanda, DuckDB
Implementing - making Coffee While cargo builds... and then lunch .... & dinner
Accessing Dave's Website battleplan.uk for style guide
Making CV Snazzy - Full Design System and Mantine
Over-engineering complete - next time use Word and the Print button
Poking hole in Docker so you can see it...
Thanking User for their time and interest
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Docker Container                                               │
│                                                                 │
│  [raw_history.json]     [Firebase REST API]                    │
│          │                       │                              │
│          └──────────┬────────────┘                             │
│                     ▼                                           │
│           [Rust Producer]  ← CV_SECRET env var               │
│           (tokio + reqwest + serde_json)                        │
│                     │                                           │
│                     ▼  cv_events topic                         │
│          ┌──────────────────────┐                              │
│          │  Redpanda (port 9092)│                              │
│          └──────────┬───────────┘                              │
│                     ▼                                           │
│           [Rust Consumer]                                       │
│           (rdkafka + duckdb crate)                             │
│                     │                                           │
│                     ▼                                           │
│              [DuckDB] cv_gold.db                               │
│                     │                                           │
│          ┌──────────▼───────────┐                              │
│          │  Axum web server     │  port 8080                   │
│          │  /api/* → DuckDB     │                              │
│          │  /*     → static     │                              │
│          └──────────┬───────────┘                              │
│                     ▼                                           │
│        [Vite + React + Mantine]  localhost:8080                │
└─────────────────────────────────────────────────────────────────┘
```

---

## Data Sources

### raw_history.json (baked into the image)
- Intentionally messy: inconsistent date formats, nested fields, legacy keys
- Contains a pointer/mapping to Firebase — not the full data
- The "...yikes!" log line earns its place here

### Firebase (fetched at runtime via REST API)
- **Project: `battleplan-dev-2024`** — the dev instance of battleplan.uk
- This is the same project that powers the site we're stealing the design from. Good story for the README.
- No official Rust Firebase SDK — use `reqwest` against the Firebase REST API directly
- Auth: Firebase service account key (JSON), passed into the container as an env var or mounted secret — never baked into the image
- Stores dynamic data: current availability, latest project highlights, tech stack config
- Enables updating CV content without rebuilding the image

**Confirmed:**
- **Firestore**
- New dedicated `cv` collection — created specifically for this project
- Service account key is embedded inside the encrypted `raw_history.json` — unlocked at runtime by `CV_SECRET`
- Service account must be locked down to the minimum viable permissions:
  - `roles/datastore.viewer` scoped to the `cv` collection only
  - No access to any other Firestore collections or Firebase services
  - This is part of the story — the container demonstrates IAM hygiene, not just "I know Firebase"





---

## Rust Crate Breakdown

```toml
[dependencies]
tokio        = { version = "1", features = ["full"] }   # async runtime
rdkafka      = "0.36"                                    # Kafka/Redpanda client
duckdb       = "1"                                       # DuckDB storage
axum         = "0.7"                                     # web server + API
serde        = { version = "1", features = ["derive"] } # serialisation
serde_json   = "1"                                       # JSON handling
reqwest      = { version = "0.12", features = ["json"] } # Firebase REST calls
tokio-stream = "0.1"                                     # async stream helpers
aes-gcm      = "0.10"                                    # AES-256-GCM encryption
pbkdf2       = "0.12"                                    # key derivation from CV_SECRET
sha2         = "0.10"                                    # PBKDF2 hash function
```

---

## Rust Binary Structure (`cv_engine`)

`main.rs` orchestrates everything in sequence:

1. Print log script lines to stdout (with small sleeps for dramatic effect)
2. Start Redpanda process as a child process
3. Spawn Tokio task: **Producer** — reads JSON + Firebase, emits events to `cv_events`
4. Spawn Tokio task: **Consumer** — reads from `cv_events`, writes to DuckDB
5. `tokio::join!` both tasks — wait for pipeline to complete
6. Start Axum server — serves API endpoints + pre-built Vite static files

---

## DuckDB Schema

```sql
CREATE TABLE fact_career_events (
    id          VARCHAR PRIMARY KEY,
    company     VARCHAR,
    title       VARCHAR,
    start_date  DATE,
    end_date    DATE,
    summary     VARCHAR,
    source      VARCHAR   -- 'json' or 'firebase'
);

CREATE TABLE dim_tech_stack (
    skill       VARCHAR PRIMARY KEY,
    category    VARCHAR,  -- 'cloud', 'languages', 'data', etc.
    level       VARCHAR   -- 'expert', 'proficient', 'familiar'
);

CREATE TABLE dim_education (
    year        INTEGER,
    qualification VARCHAR,
    institution VARCHAR
);

CREATE TABLE fact_pipeline_run (
    run_id          VARCHAR,
    events_produced INTEGER,
    events_consumed INTEGER,
    duration_ms     INTEGER,
    ran_at          TIMESTAMP
);
```

---

## Axum API Endpoints

| Endpoint | Returns |
|---|---|
| `GET /api/experience` | Career events from DuckDB |
| `GET /api/skills` | Tech stack from DuckDB |
| `GET /api/education` | Education from DuckDB |
| `GET /api/pipeline` | Pipeline run health metrics |
| `GET /*` | Static Vite build files |

---

## Frontend: Vite + React + Mantine

Pre-built at Docker build time. Served as static files by Axum.

Pages:
- **Pipeline Health** — ingestion log ticker, events processed, Redpanda stream status badge
- **Experience** — job history from `/api/experience`
- **Education** — from `/api/education`

Styled after battleplan.uk. No PDF export (dropped — Playwright is a container nightmare).

---

## Dockerfile (Multi-stage)

```dockerfile
# Stage 1: Build Rust binary
FROM rust:1.78-slim AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Stage 2: Build Vite frontend
FROM node:20-slim AS node-builder
WORKDIR /frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend ./
RUN npm run build

# Stage 3: Production image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Non-root user — non-negotiable
RUN useradd -m -u 1001 dave
USER dave
WORKDIR /cv

# Copy compiled binary
COPY --from=rust-builder /app/target/release/cv_engine ./cv_engine

# Copy pre-built frontend
COPY --from=node-builder /frontend/dist ./static

# Copy Redpanda binary (downloaded in rust-builder stage)
COPY --from=rust-builder /usr/local/bin/redpanda /usr/local/bin/redpanda

# Copy data
COPY raw_history.json ./

EXPOSE 8080
ENTRYPOINT ["./cv_engine"]
```

---

## Distribution

### GHCR
```bash
docker pull ghcr.io/tyberium/cv:latest
```
Image appears in the GitHub repo sidebar under "Packages".

### GitHub Actions (`.github/workflows/publish-cv.yml`)

Triggers on push to `main`:
1. `cargo test` — unit tests
2. Build Docker image (multi-stage)
3. Push to GHCR using `${{ secrets.GITHUB_TOKEN }}`

---

## File Structure

```
cv-pipeline/
├── Dockerfile
├── docker-compose.yml
├── README.md                     ← recruiter instructions
├── raw_history.json              ← messy source data (baked into image)
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs                   ← orchestration + log script
│   ├── producer.rs               ← JSON + Firebase → Redpanda
│   ├── consumer.rs               ← Redpanda → DuckDB
│   ├── db.rs                     ← DuckDB schema + queries
│   ├── api.rs                    ← Axum routes
│   └── models.rs                 ← serde structs
├── tests/
│   └── pipeline_tests.rs         ← integration tests
├── frontend/
│   ├── src/
│   │   ├── App.tsx
│   │   ├── pages/
│   │   │   ├── PipelineHealth.tsx
│   │   │   ├── Experience.tsx
│   │   │   └── Education.tsx
│   │   └── components/
│   ├── package.json
│   └── vite.config.ts
└── .github/
    └── workflows/
        └── publish-cv.yml
```

---

## Development Order (Frontend-First)

1. **Vite + Mantine frontend** with hardcoded mock data — lock in the UI and target schema
2. **Axum server** serving mock JSON + static files — confirm frontend wiring
3. **DuckDB schema** — reverse-engineered from what the frontend actually queries
4. **`consumer.rs`** — reads events, writes to DuckDB, replaces mock data
5. **`producer.rs`** — JSON ingestion + Firebase REST fetch, emits to Redpanda
6. **Redpanda** — wire up the event stream between producer and consumer
7. **`main.rs`** — log script output, orchestration, process management
8. **Dockerfile** — multi-stage build, confirm image size is lean
9. **Tests** — `cargo test` + integration tests
10. **GitHub Actions** — CI/CD, push to GHCR

---

## PostHog Source Code Integration (The Showboating)

Three specific pieces lifted directly from PostHog's open source Rust codebase
(`posthog/rust/capture/src/`). Each is unnecessary. That's the point.

### 1. `validate_token` — from `token.rs`
PostHog runs this function on every single event capture request in production.
We use it to validate `CV_SECRET` before attempting decryption.

Lifted verbatim, with an attribution comment:

```rust
// Borrowed from PostHog/posthog: rust/capture/src/token.rs
// Used in production to validate every event capture token.
// This isnt exactly necessary in this codebase, but I thought you'd recognise it....
pub fn validate_token(token: &str) -> Result<(), InvalidTokenReason> {
    if token.is_empty() { return Err(InvalidTokenReason::Empty); }
    if token.len() > 64 { return Err(InvalidTokenReason::TooLong); }
    if !token.is_ascii() { return Err(InvalidTokenReason::NotAscii); }
    if token.starts_with("phx_") { return Err(InvalidTokenReason::PersonalApiKey); }
    if token.contains('\0') { return Err(InvalidTokenReason::NullByte); }
    Ok(())
}
```

Log line this unlocks: `"Validating token structure... [PostHog capture pattern]"`

### 2. PostHog Event Schema — Kafka payload format
Every career milestone the producer emits to Redpanda is formatted in the exact
PostHog capture event schema. An engineer at PostHog opens the code and immediately
recognises their own event structure flowing through the pipeline.

```json
{
  "event": "career_milestone",
  "distinct_id": "dave_carroll",
  "properties": {
    "company": "Wealth Wizards",
    "title": "Senior Data Engineer",
    "achievement": "25% AWS cost reduction",
    "duration_months": 18
  },
  "timestamp": "2023-01-01T00:00:00Z",
  "token": "cv_pipeline_v1"
}
```

This means `cv_events` on Redpanda is a valid PostHog-format event stream.

## Resolved Decisions

### battleplan.uk Design System
Lift the design system directly out of battleplan.uk — components, colour tokens, typography, spacing, the lot.
The irony of using your own website's design system to present your CV is the point.
First step in frontend build: inspect battleplan.uk and extract the Mantine theme config and any custom tokens.

### CV_SECRET — Symmetric Encryption
The secret is a plaintext string provided to the recruiter.
`raw_history.json` is **symmetrically encrypted** at rest inside the image (not hashed — hashing is one-way and irreversible; this needs to be decryptable).

Implementation:
- Derive a 256-bit key from `CV_SECRET` using **PBKDF2-HMAC-SHA256** with a fixed salt baked into the binary
- Encrypt `raw_history.json` using **AES-256-GCM** (authenticated encryption — detects tampering)
- At runtime: derive key → decrypt JSON in memory → proceed with pipeline
- Rust crates: `aes-gcm`, `pbkdf2`, `sha2`
- The encrypted file is what ships in the image. Without the correct `CV_SECRET`, the container starts, logs "Decrypting..." and then stops. Nothing leaks.

### Redpanda — Single Image (No Sidecar)
Redpanda runs as a child process inside the single container, managed by the Rust entrypoint.
No docker-compose, no inter-container networking, no multi-service complexity.

The Dockerfile will include a comment block explaining the trade-off:
```dockerfile
# NOTE: Redpanda would typically run as a dedicated sidecar service (see docker-compose.yml.example).
# For recruiter UX, we embed it here as a managed child process — one image, one command, no networking gubbins.
# The architectural pattern is sound; this is a deliberate DX simplification.
```

This is the "over-engineering complete" move — you *know* the right pattern, you documented it, and you made a conscious pragmatic call. That's senior thinking.
