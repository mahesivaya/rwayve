# syntax=docker/dockerfile:1.7
# Requires BuildKit. With docker-compose v2 it's enabled by default; if you
# use the legacy `docker-compose`, prefix builds with `DOCKER_BUILDKIT=1`.

# ---- Builder ----------------------------------------------------------------
FROM rust:1.95-slim-bookworm AS builder

WORKDIR /app

# pkg-config helps a few crates probe system libs even with rustls-tls; keep
# the apt set minimal to keep this layer small and cache-friendly.
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Step 1: prime the dependency cache with a stub crate. This makes the heavy
# deps (actix-web, sqlx, tracing-*, reqwest, ...) compile once and stay cached
# in BuildKit's mount; later edits to src/ no longer recompile them.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release && rm -rf src

# Step 2: copy real sources and rebuild. Only our crate recompiles — deps
# are already in the cache mount.
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    touch src/main.rs \
    && cargo build --release \
    && cp /app/target/release/rwayve /app/rwayve.bin

# ---- Runtime ----------------------------------------------------------------
# Slim runtime — tens of MB instead of ~1.5 GB for the full rust image.
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/rwayve.bin /app/rwayve

# Default RUST_LOG; override via env. Per-target file layers ignore this.
ENV RUST_LOG=info
EXPOSE 8080
CMD ["/app/rwayve"]
