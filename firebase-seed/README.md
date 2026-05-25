# Firebase seed (Phase 6)

Populates the `cv` collection in `battleplan-dev-2024` Firestore:

| Document       | Purpose                                      |
|----------------|----------------------------------------------|
| `availability` | Live availability on `/api/profile`            |
| `skills`       | Skill levels/categories (pipeline + `/api/skills`) |
| `highlights`   | Current-role project call-outs (`ww_001`)    |

Schema: `planning/data_models.md` §5.

## 1. Seed documents (write access)

Uses your **user** credentials via gcloud (needs Firestore write, e.g. Owner/Editor):

```powershell
gcloud auth login
gcloud config set project battleplan-dev-2024

# Preview payloads
cargo run --bin firebase_seed -- --dry-run

# Write to Firestore
cargo run --bin firebase_seed
```

Edit `firebase-seed/data/*.json` and re-run to update.

Alternative auth: set `GOOGLE_APPLICATION_CREDENTIALS` to a service account JSON with write access.

## 2. Read-only key for the pipeline

The running CV container only needs read access. Create `cv-reader`:

```powershell
.\scripts\firebase\create-cv-reader.ps1
```

```bash
chmod +x scripts/firebase/create-cv-reader.sh
./scripts/firebase/create-cv-reader.sh
```

Output: `firebase-seed/out/cv-reader-key.json` (gitignored).

Copy the JSON into `raw_history_plain.json` → `firebase_credentials`, then re-encrypt with `cargo run --bin encrypt`. See `README.md` for the full copy/photo workflow.

`roles/datastore.viewer` is project-scoped; the Rust producer only requests `cv/*` document paths.
