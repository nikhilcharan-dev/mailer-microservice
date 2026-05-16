# syntax=docker/dockerfile:1.7
#
# Multi-stage build for the central-mailer workspace.
#
#   target=backend   → final image runs the Axum API server
#   target=frontend  → final image runs the Leptos SSR dashboard
#
# Dependency builds are cached via cargo-chef so source-only changes do not
# trigger a full re-compile of the dependency tree (~200 crates).

ARG RUST_VERSION=1.83

# ── chef ────────────────────────────────────────────────────────────────────
# Shared image with cargo-chef installed.
FROM rust:${RUST_VERSION}-bookworm AS chef
RUN cargo install cargo-chef --locked --version ^0.1
WORKDIR /build

# ── planner ────────────────────────────────────────────────────────────────
# Walks Cargo.toml/Cargo.lock and writes recipe.json describing the dep tree.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── builder ────────────────────────────────────────────────────────────────
# Builds all workspace dependencies in a cached layer (the recipe.json layer
# is only invalidated when Cargo.lock changes), then compiles the workspace.
FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --workspace
COPY . .
RUN cargo build --release --bin central-mailer --bin frontend

# ── runtime-base ──────────────────────────────────────────────────────────
# Slim Debian with TLS roots, libssl3 (native-tls for lettre), curl (healthcheck).
FROM debian:bookworm-slim AS runtime-base
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        curl \
        tini \
 && rm -rf /var/lib/apt/lists/*
# Drop privileges by default
RUN groupadd --system --gid 1001 app \
 && useradd  --system --uid 1001 --gid app --no-create-home --shell /usr/sbin/nologin app
WORKDIR /app

# ── backend ────────────────────────────────────────────────────────────────
FROM runtime-base AS backend
COPY --from=builder /build/target/release/central-mailer /usr/local/bin/central-mailer
USER app
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=3 \
    CMD curl --fail --silent --show-error http://127.0.0.1:8080/ready || exit 1
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/central-mailer"]

# ── frontend ───────────────────────────────────────────────────────────────
FROM runtime-base AS frontend
COPY --from=builder /build/target/release/frontend /usr/local/bin/frontend
# The frontend reads ServeDir("frontend/static") relative to its workdir.
COPY --chown=app:app frontend/static /app/frontend/static
USER app
EXPOSE 3000
HEALTHCHECK --interval=15s --timeout=5s --start-period=5s --retries=3 \
    CMD curl --fail --silent --show-error http://127.0.0.1:3000/login || exit 1
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/frontend"]
