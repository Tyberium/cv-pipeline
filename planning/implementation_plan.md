# Implementation Plan: CV Pipeline

---

## Status Key

- ✅ Done
- 🔲 Not started
- ⚠️ Known gotcha — see notes

---

## Phase 0 — Design & Scaffold ✅

- ✅ Architecture finalised (`planning/final_plan.md`)
- ✅ Data models designed working backwards from frontend (`planning/data_models.md`)
  - 9 DuckDB tables defined
  - 6 API routes defined (profile, profile/photo, experience, skills, education, pipeline)
  - 6 Redpanda event types catalogued
  - Firebase collection structure defined
  - `raw_history.json` structure + intentional messiness specified
  - All open questions resolved
- ✅ CI/CD workflow designed (`planning/cicd.md`)
  - 3-job pipeline: test → build-push → smoke-test
  - Only `CV_SECRET` (GitHub Actions secret) + built-in `GITHUB_TOKEN` for GHCR push
  - Firebase credentials stay encrypted inside the image — never in the repo
- ✅ Cursor rules written (`.cursor/rules/`)
  - `project-overview.mdc` — context, stack, dev order (always on)
  - `no-legacy.mdc` — no backwards compat shims, delete and move on (always on)
  - `timestamps.mdc` — single conversion point per layer, ISO strings at all boundaries (always on)
  - `rust-standards.mdc` — file ownership, error handling, no code sprawl (`src/**/*.rs`)
  - `data-pipeline.mdc` — event shapes, consumer routing, DuckDB rules (`producer/consumer/db`)
  - `frontend-standards.mdc` — pages, types, data fetching, Mantine only (`frontend/**`)
- ✅ Git repo initialised, `.gitignore`, `.gitattributes`
- ✅ `Cargo.toml` — all dependencies pinned, `cv_engine` + `encrypt` + `firebase_seed` binaries declared
- ✅ Rust source stubs — `main.rs`, `models.rs`, `db.rs`, `api.rs`, `producer.rs`, `consumer.rs`, `src/bin/encrypt.rs`
  - `models.rs` is fully written: `CvEvent` PostHog envelope, all `Raw*` input types, all `Clean*` output types
  - `db.rs` schema SQL is fully written; insert/query functions are stubs (`todo!()`)
  - All other files are responsibility-documented stubs
- ✅ Vite + React + TypeScript frontend scaffolded
  - `src/types.ts` — all API response types, single source of truth
  - `src/utils/dates.ts` — the only place `new Date()` lives in the frontend
  - `src/theme.ts` — Mantine theme derived from wireframe colour tokens
  - `src/pages/` — 5 page stubs (About, Experience, Skills, Education, PipelineHealth)
    - All pages make real fetch calls from day one
    - All pages have proper loading / empty states (no mock data)
  - `vite.config.ts` — `/api` proxy → `:8080`, build output → `../static`
- ✅ `README.md` — recruiter-facing, one command
- ✅ `docker-compose.yml`
- ✅ Initial commits on `main`

---

## Phase 1 — Axum API + DuckDB (make the frontend do something) ✅

**Goal:** `cargo run` starts a server, frontend fetch calls return empty-but-correct JSON, loading states resolve.

- ✅ `db.rs` — all `insert_*` and `query_*` functions implemented
- ✅ `api.rs` — Axum router (profile, profile/photo, experience, skills, education, pipeline)
- ✅ `main.rs` — minimal startup: `db::open()`, `db::init_schema()`, Axum on `:8080`
- ✅ `build.rs` — links `rstrtmgr.lib` on Windows (DuckDB requirement)
- ✅ Verified: all 5 endpoints respond correctly with empty/null shapes

---

## Phase 2 — Frontend UI (BattlePlan theme) ✅

**Goal:** Pages look like the CV; real API fetches; BattlePlan dark sidebar layout.

- ✅ `AppShell` sidebar nav (About, Employment, Skills, Education, Pipeline)
- ✅ `About` — hero (name, title, contact, availability, encrypted photo via `/api/profile/photo`), About Me + Why PostHog (`CopyBlock` paragraphs)
- ✅ `Experience` — timeline, responsibilities, key projects
- ✅ `Skills` — category cards, level pills
- ✅ `Education` — formal + certifications columns
- ✅ `PipelineHealth` — run stats, source/event breakdown
- ✅ `EmptyState` — loading / empty pipeline messaging
- ✅ `theme.ts` — `BP` tokens; `dates.ts` — sole `new Date()` usage

---

## Phase 3 — DuckDB Consumer (events → storage) ✅

**Goal:** Consumer event routing implemented and fully tested against in-memory DuckDB.

- ✅ `consumer::process_event` — routes all 7 event types to correct `db::insert_*` function
- ✅ `db::insert_*` functions — all implemented with parameterised SQL (done in Phase 1)
- ✅ 5 inline tests (`#[cfg(test)]`) — profile, experience, skills, education, unknown event type
- ✅ `cargo test` — 5/5 passing in 0.22s
- Note: `consumer::run()` (rdkafka loop) stays as `todo!()` until Phase 5

---

## Phase 4 — Producer (JSON + Firebase → Redpanda) ✅

**Goal:** Producer reads and normalises `raw_history.json`, emits events. Full pipeline runs short-circuit (no Redpanda yet).

- ✅ `src/crypto.rs` + `src/bin/encrypt.rs` — PBKDF2-HMAC-SHA256 + AES-256-GCM (shared by `raw_history` and `photo`)
- ✅ `raw_history_plain.json` — authored from `copy.md` with intentional messiness (gitignored)
- ✅ `raw_history.json` — encrypted output committed (encrypt with `CV_SECRET`; see `README.md`)
- ✅ `photo.enc` — encrypted headshot committed; `photo.png` gitignored (local only). `GET /api/profile/photo` after startup decrypt
- ✅ `producer::decrypt` — decrypts in memory, safe error message if CV_SECRET wrong
- ✅ `producer::produce` — all 59 events produced from 8 jobs + 20 education entries
  - `parse_date_flexible` handles 5 formats including `%b-%y` 2-digit year
  - Bug fixed: chrono's flexible whitespace let `%B %Y` consume separator and sign-read `-22` as year −22; resolved by manual split-and-normalise before any chrono format call
  - Responsibilities: string (`\n`-split) and array handled via `normalise_responsibilities`
  - `achievements` field (Envelop Risk) normalised same way
  - Key projects: 3 different key names (`projects`, `key_deliverables`, `achievements`) handled
- ✅ Firebase fetch stubbed — gracefully skips if credentials are placeholder, warns in log
- ✅ `main.rs` updated — full Phase 4 startup: decrypt → produce → consume (short-circuit) → Axum
- ✅ `validate_token` — borrowed from PostHog/posthog:rust/capture/src/token.rs
- ✅ `db::insert_pipeline_event_log` added — event type breakdown for Pipeline Health page
- ✅ 6 tests passing — 5 consumer round-trips + 1 date parsing suite (9 format cases)
- ✅ Frontend shows real data: all 8 jobs, 20 education entries, full profile
- ✅ `producer::emit()` + `consumer::run()` — Redpanda path (Phase 5)

---

## Phase 5 — Redpanda Integration ✅ (Docker e2e pending confirmation)

**Rationale:** Redpanda is Linux-only. Wiring it on Windows is pain with no payoff — the
final target is a Linux Docker container. Phase 8 Dockerfile is the test harness for this.

**Goal:** Redpanda runs as a child process, producer and consumer use it as the message bus.

- ✅ Source Redpanda binary for the Docker image (25.2.7 amd64, S3 release)
- ✅ `config/redpanda.yaml` — broker config extracted from `main.rs`
- ✅ `main.rs` — spawn Redpanda as `std::process::Command` child process (Redpanda 25.x YAML mode)
- ✅ `main.rs` — poll for Redpanda readiness before starting producer
- ✅ `producer.rs` — `emit()` sends events to `cv_events` topic (`rdkafka`, `kafka` feature flag)
- ✅ `consumer.rs` — `run()` loop reads from Redpanda, calls `process_event` per message
- ✅ Create `cv_events` topic programmatically via Kafka admin API (`rdkafka`)
- 🔲 End-to-end test: `docker run` full pipeline, verify all tables populated, frontend shows real data
- ✅ Local Windows short-circuit path verified (78 events, Firebase skills + live availability)

---

## Phase 6 — Firebase (live fetch) ✅

**Goal:** `/api/profile` returns live `availability` from Firestore on each request. Producer ingests `cv/skills` and `cv/highlights` during the pipeline run.

### Firestore infra ✅

- ✅ `cv` collection created in `battleplan-dev-2024` Firestore
  - Documents: `availability`, `skills`, `highlights` — see `data_models.md` §5
  - Populated from `planning/copy.md` via `firebase-seed/data/*.json`
- ✅ `src/bin/firebase_seed.rs` — dev utility to upsert `cv/*` documents (Firestore REST)
- ✅ `scripts/firebase/create-cv-reader.ps1` + `.sh` — create `cv-reader@battleplan-dev-2024` SA
- ✅ `cv-reader` granted `roles/datastore.viewer` (project-scoped; app reads `cv/*` only)
- ✅ Read auth verified — `runQuery` against `cv` collection using service account key
- ✅ Real `firebase_credentials` merged into `raw_history_plain.json` and re-encrypted

### Runtime wiring ✅

- ✅ `gcp_auth` crate — `CustomServiceAccount::from_json` + `TokenProvider::token` (replaces hand-rolled JWT)
- ✅ `producer.rs` — Firestore REST fetch `cv/skills` + `cv/highlights` → `skill_added` / `project_highlight` events (19 events)
- ✅ `api.rs` — live `cv/availability` fetch on every `/api/profile` request; merged into response
- ✅ `main.rs` — passes `firebase_credentials` to `api::router`; counts `firebase_events` in pipeline telemetry
- ✅ Verified locally: availability returns `{ available: true, from_date: "2026-08-01", message: "..." }`

---

## Phase 7 — Log Script + Orchestration ✅

**Goal:** Terminal UX plays out before the web server starts. The pipeline *is* the CV.

- ✅ `main.rs` — personality `println!` lines + structured logs to `/tmp/cv_pipeline.log`
- ✅ `validate_token` — borrowed from PostHog `rust/capture/src/token.rs`
- ✅ Startup: validate → decrypt `raw_history.json` + `photo.enc` → pipeline → Axum on `:8080`
- ✅ Redpanda child process on Docker/`kafka` feature path; short-circuit on Windows dev

---

## Phase 8 — Dockerfile 🔲 (build verified locally; GHCR publish pending)

**Goal:** `docker run -p 8080:8080 -e CV_SECRET=<secret> ghcr.io/tyberium/cv:latest` works.

- ✅ Multi-stage Dockerfile written (rust-builder → node-builder → debian:bookworm-slim)
- ✅ `.dockerignore` written — excludes `target/`, `node_modules/`, `cv_gold.db*`, `raw_history_plain.json`, `photo.png`, `planning/`
- ✅ Non-root user `dave` (uid 1001)
- ✅ Redpanda binary copied into runtime image (`/opt/redpanda`)
- ✅ `config/redpanda.yaml` copied at build time for `include_str!`
- ✅ Docker Desktop installed (29.4.3)
- 🔲 `docker build -t cv:local .` — verify build succeeds (pending daemon)
- 🔲 `docker run -p 8080:8080 -e CV_SECRET=<secret> cv:local` — verify end-to-end
- ✅ `raw_history.json` + `photo.enc` (encrypted) baked into image — in COPY
- 🔲 Verify image size is lean (`docker images cv:local`)
- 🔲 Docker e2e — Redpanda + Kafka path, all API routes incl. `/api/profile/photo`, live availability
- ⚠️ Image size — Redpanda binary dominates (~multi-GB); acceptable trade-off for single-command UX

---

## Phase 9 — Tests ✅

- ✅ Consumer round-trips + date parsing (`producer::tests`)
- ✅ `validate_token` — empty, too short, happy path (`main.rs` tests)
- ✅ `cargo test` — 9 tests on `cv_engine` (CI runs `cargo test` without `kafka` feature)

---

## Phase 10 — CI/CD 🔲 (workflow in repo; GHCR + cold pull TBC)

- ✅ `.github/workflows/publish-cv.yml` — test → build-push → smoke-test (see `planning/cicd.md`)
- 🔲 `CV_SECRET` in GitHub repo secrets (must match encrypted blobs in repo)
- 🔲 Push to `main` → image on `ghcr.io/tyberium/cv:latest`
- 🔲 GHCR package visibility Public
- 🔲 Cold `docker pull` + run — recruiter path confirmed
- 🔲 Smoke test: add `/api/profile/photo` (PNG bytes) optional check

---

## Deferred / optional

- 🔲 **History squash** — remove any early plaintext photo/secret from git history before calling repo public

---

## Known Gotchas

### ⚠️ Firebase REST Auth

The Firebase REST API requires a short-lived OAuth2 bearer token — not the service account key directly.

Handled by `gcp_auth` crate: `CustomServiceAccount::from_json` + `TokenProvider::token` with `https://www.googleapis.com/auth/datastore` scope. Shared via `producer::bearer_token()` for both pipeline fetch and live `/api/profile` availability.

### ⚠️ Firestore IAM scope

GCP IAM cannot scope `roles/datastore.viewer` to a single collection. The `cv-reader` service account has project-level read access; the application only requests `cv/*` document paths.

### ⚠️ Local Windows dev

`cargo run --bin cv_engine` uses the short-circuit path (no `kafka` feature). Full pipeline including Firebase works on Windows. Docker build uses `--features kafka` for the Redpanda message bus.

### ⚠️ Stale processes on port 8080

If `cv_engine` is started multiple times during dev, old processes can keep port 8080 and serve an outdated binary. Kill with `Stop-Process -Name cv_engine -Force` before re-testing.

### ⚠️ Redpanda as Child Process

Redpanda is a full broker binary, not a simple CLI tool. Running it as a child process requires:
1. A config file (at minimum: `kafka_api` address, `data_directory`)
2. Writing that config to a temp directory at startup
3. Polling port 9092 for readiness before the producer starts
4. Handling the child process exit in `main.rs`

The Dockerfile also needs to source the correct Redpanda binary for `linux/amd64`. ✅ Done — Redpanda 25.2.7 fetched in rust-builder stage, config in `config/redpanda.yaml`.

### ⚠️ `raw_history_plain.json` must never be committed

The `.gitignore` blocks it. Double-check before any commit touching the data files.
`raw_history.json` and `photo.enc` (encrypted) ARE committed and baked into the image.
`photo.png` and `raw_history_plain.json` must never be committed. See `README.md`.

### ⚠️ Never put secrets in `planning/copy.md`

`copy.md` is committed. Use `<provided-on-request>` in docker examples. Rotate and re-encrypt if a real passphrase was ever committed.
