# ── Stage 1: Build Rust binary ───────────────────────────────────────────────
FROM rust:1-slim-bookworm AS rust-builder

# DuckDB (bundled) compiles from source and needs a C++ compiler + make.
# rdkafka uses mklove (default build — no cmake needed).
# OpenSSL dev headers are required by reqwest (native-tls).
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev g++ make \
    && rm -rf /var/lib/apt/lists/*

# DuckDB's C++ build is memory-hungry — cap parallel jobs.
ENV CARGO_BUILD_JOBS=2

WORKDIR /build

# Layer-cache the dependency compile separately from the source compile.
# Cargo needs a compilable src/ to resolve deps — stub main and lib files satisfy it.
COPY Cargo.toml build.rs ./
# Cargo.lock is gitignored locally — generate inside the container if not copied.
COPY Cargo.lock* ./
RUN cargo generate-lockfile

RUN mkdir -p src/bin \
    && echo 'fn main() {}' > src/main.rs \
    && echo 'fn main() {}' > src/bin/encrypt.rs \
    && cargo build --release --features kafka --bin cv_engine 2>/dev/null || true \
    && rm -rf src

# Now copy real source and do the real build
COPY config ./config
COPY src ./src
RUN touch src/main.rs \
    && cargo build --release --features kafka --bin cv_engine \
    && strip /build/target/release/cv_engine

# Official broker image (~260 MB /opt/redpanda). The S3 tarball is the same version
# but ships an unstripped 1.8 GB binary — that was the bulk of the 5 GB image.
FROM redpandadata/redpanda:v25.2.7 AS redpanda

# ── Stage 2: Build Vite frontend ─────────────────────────────────────────────
FROM node:20-slim AS node-builder

WORKDIR /frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

COPY frontend ./
# outDir is ../static (see vite.config.ts) so output lands at /static
RUN npm run build

# ── Stage 3: Runtime image ────────────────────────────────────────────────────
#
# NOTE: Redpanda would typically run as a dedicated sidecar service.
# For recruiter UX it will run as a managed child process here — one image,
# one command, no networking gubbins. The architectural pattern is sound;
# this is a deliberate DX simplification. Redpanda wiring is Phase 5.
#
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 ca-certificates libatomic1 libstdc++6 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1001 dave

# Redpanda — broker + bundled libs from the official image (not the fat S3 tarball).
COPY --from=redpanda /opt/redpanda /opt/redpanda
RUN ln -sf /opt/redpanda/bin/redpanda /usr/local/bin/redpanda

USER dave
WORKDIR /cv

COPY --from=rust-builder /build/target/release/cv_engine ./cv_engine
COPY --from=node-builder /static ./static
COPY raw_history.json photo.enc ./

EXPOSE 8080
ENTRYPOINT ["./cv_engine"]
