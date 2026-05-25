# CI/CD Design: CV Pipeline

---

## Trigger

| Event | Behaviour |
|---|---|
| Push to `main` | Run tests → build image → push to GHCR |
| Pull request to `main` | Run tests + build only (no push) |

No other branches are built. No manual dispatch needed.

---

## Secrets

Only **two secrets** are needed. `CV_SECRET` lives in **GitHub** repository secrets (masked); `GITHUB_TOKEN` is built-in for GHCR push.

| Secret | Value | Used for |
|---|---|---|
| `GITHUB_TOKEN` | Built-in (GitHub provides automatically) | GHCR authentication — no setup needed |
| `CV_SECRET` | The decryption passphrase | Running the container in the smoke test step |

### Why Firebase credentials are NOT a separate secret

The Firebase service account key is encrypted inside `raw_history.json`,
which ships inside the image. `CV_SECRET` decrypts it at runtime.

CI doesn't need to know about Firebase at all. The only secret it needs
is `CV_SECRET` — and that's only required for the smoke test step
(running the built container to verify it actually starts).

**To set up (GitHub):**
```
Repository → Settings → Secrets and variables → Actions → New repository secret
Name: CV_SECRET
Value: <same passphrase used for cargo run --bin encrypt>
```

Must match the key used to produce committed `raw_history.json` and `photo.enc`.

---

## Jobs

```
push to main
    │
    ▼
┌─────────────────┐
│   test          │  ubuntu-latest
│  ─────────────  │  cargo test
│  cargo test     │  frontend build check (catches TS errors)
│  npm run build  │
└────────┬────────┘
         │ success
         ▼
┌─────────────────┐
│  build-push     │  ubuntu-latest
│  ─────────────  │  docker buildx (multi-platform optional)
│  docker build   │  push to ghcr.io/tyberium/cv
│  docker push    │  tags: latest + sha-{short}
└────────┬────────┘
         │ success
         ▼
┌─────────────────┐
│  smoke-test     │  ubuntu-latest
│  ─────────────  │  pull built image
│  docker run     │  run with CV_SECRET
│  curl /api/*    │  verify endpoints respond
└─────────────────┘
```

PRs only run the `test` job (no push, no smoke test).

---

## Image Tags

Every push to `main` produces two tags:

| Tag | Example | Purpose |
|---|---|---|
| `latest` | `ghcr.io/tyberium/cv:latest` | What recruiters run — always the current build |
| `sha-{short}` | `ghcr.io/tyberium/cv:sha-a1b2c3d` | Traceability — pin to a specific commit if needed |

No semver tags. This project has no versioning story — `latest` is correct.

---

## Workflow File

`.github/workflows/publish-cv.yml`

```yaml
name: Build and Publish CV

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ghcr.io/tyberium/cv

jobs:

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust build
        uses: Swatinem/rust-cache@v2

      - name: Run Rust unit tests
        run: cargo test

      - name: Set up Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
          cache-dependency-path: frontend/package-lock.json

      - name: Frontend build check
        working-directory: frontend
        run: |
          npm ci
          npm run build

  build-and-push:
    name: Build and Push Image
    needs: test
    runs-on: ubuntu-latest
    # Only push on main — PRs build but don't push
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    permissions:
      contents: read
      packages: write
    outputs:
      image-digest: ${{ steps.build.outputs.digest }}
      image-tag: ${{ steps.meta.outputs.version }}
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GHCR
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract image metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.IMAGE_NAME }}
          tags: |
            type=raw,value=latest
            type=sha,prefix=sha-,format=short

      - name: Build and push
        id: build
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          # Layer cache stored in GitHub Actions cache — speeds up subsequent builds
          cache-from: type=gha
          cache-to: type=gha,mode=max

  smoke-test:
    name: Smoke Test
    needs: build-and-push
    runs-on: ubuntu-latest
    steps:
      - name: Pull built image
        run: docker pull ${{ env.IMAGE_NAME }}:latest

      - name: Start container
        run: |
          docker run -d \
            --name cv_smoke \
            -p 8080:8080 \
            -e CV_SECRET=${{ secrets.CV_SECRET }} \
            ${{ env.IMAGE_NAME }}:latest

      - name: Wait for server to be ready
        run: |
          for i in $(seq 1 30); do
            if curl -sf http://localhost:8080/api/pipeline; then
              echo "Server is up"
              exit 0
            fi
            echo "Waiting... ($i/30)"
            sleep 2
          done
          echo "Server did not start in time"
          docker logs cv_smoke
          exit 1

      - name: Smoke test API endpoints
        run: |
          curl -sf http://localhost:8080/api/profile   | jq '.name'
          curl -sf http://localhost:8080/api/experience | jq 'length'
          curl -sf http://localhost:8080/api/skills     | jq 'keys'
          curl -sf http://localhost:8080/api/education  | jq 'length'
          curl -sf http://localhost:8080/api/pipeline   | jq '.events_consumed'
          curl -sf -o /dev/null -w "%{http_code}" http://localhost:8080/api/profile/photo | grep -q 200

      - name: Tear down
        if: always()
        run: docker rm -f cv_smoke
```

---

## Build Cache

The workflow uses GitHub Actions cache (`type=gha`) for Docker layer caching.
The multi-stage build benefits significantly from this:

- **Rust stage**: `cargo build --release` is the slow step (~3-5 min cold).
  Cached if `Cargo.lock` and `src/` haven't changed.
- **Node stage**: `npm ci` + `npm run build` is fast.
  Cached if `frontend/package-lock.json` hasn't changed.
- **Final stage**: Always fast — just `COPY` commands.

Expected cold build: ~6-8 min. Warm build (deps unchanged): ~2-3 min.

---

## Image Visibility

By default GHCR packages are private. To make the image publicly pullable
(so recruiters don't need to authenticate):

```
GitHub repo → Packages → cv → Package settings → Change visibility → Public
```

This only needs doing once after the first push. After that, recruiters can run:
```bash
docker run -p 8080:8080 -e CV_SECRET=<provided-on-request> ghcr.io/tyberium/cv:latest
```
with no login required.

---

## What Does NOT Go in CI

| Thing | Why not |
|---|---|
| Firebase service account JSON | Encrypted inside `raw_history.json` — CI never sees it |
| `raw_history.json` plaintext | Never exists outside of runtime memory — only the encrypted file is in the repo |
| Any content secrets | All content is either in the encrypted JSON or in Firestore |
