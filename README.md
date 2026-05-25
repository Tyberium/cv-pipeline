# My CV is a Docker image.

```bash
docker run -p 8080:8080 -e CV_SECRET=<provided-on-request> ghcr.io/tyberium/cv:latest
```

Then open [http://localhost:8080](http://localhost:8080).

---

## What happens when you run it

A pipeline plays out in the terminal before the web server starts.
That's not a loading spinner — it's the CV.

Wrong or missing `CV_SECRET` → the container exits during decrypt. No site, no leaked plaintext.

---

## Local dev

Secrets and plaintext assets are **not** in the repo.

1. Copy `.env.example` to `.env` (gitignored) and set `CV_SECRET`.
2. Keep `raw_history_plain.json` and `photo.png` at the repo root (both gitignored).
3. Run the engine:

```bash
# PowerShell
$env:CV_SECRET = "<your secret>"
cargo run --bin cv_engine
```

Re-encrypt after changing plaintext sources:

```bash
cargo run --bin encrypt   # writes raw_history.json + photo.enc (commit those)
```

Optional frontend dev server (proxies `/api` to `:8080`; engine must be running first):

```bash
cd frontend && npm run dev
```

**Updating copy:** draft in `planning/copy.md`, merge into `raw_history_plain.json`, then `cargo run --bin encrypt`. Do not commit plaintext JSON or `photo.png`.

---

## docker-compose

Create `.env` next to `docker-compose.yml` with `CV_SECRET=<secret>`, then:

```bash
docker compose up
```

Uses `ghcr.io/tyberium/cv:latest` when published. For a local build: `docker build -t cv:local .` and set `image: cv:local` in `docker-compose.yml`.

---

## Stack

| Layer | Technology |
|---|---|
| Ingestion | Rust + tokio |
| Message bus | Redpanda (Kafka path in Docker only) |
| Storage | DuckDB |
| API | Axum |
| Frontend | Vite + React + Mantine |

One Docker image. One command. No dependencies on your machine except Docker.

---

## Docs (repo)

| Doc | Purpose |
|-----|---------|
| `planning/data_models.md` | API shapes, DuckDB schema, events |
| `planning/implementation_plan.md` | Build status by phase |
| `planning/cicd.md` | GitHub Actions + GHCR |
| `planning/copy.md` | CV prose draft (not loaded by the pipeline) |
