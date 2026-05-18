# Multi-stage build for the cloudMailer workspace.
#
#   target=backend   → final image runs the Axum API server
#   target=frontend  → final image runs the Leptos SSR dashboard

# ── chef ────────────────────────────────────────────────────────────────────
FROM rust:bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /build

# ── planner ────────────────────────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── builder ────────────────────────────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --workspace
COPY . .
RUN cargo build --release --bin cloud-mailer --bin frontend

# ── runtime-base ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime-base
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        curl \
        tini \
 && rm -rf /var/lib/apt/lists/*
RUN groupadd --system --gid 1001 app \
 && useradd  --system --uid 1001 --gid app --no-create-home --shell /usr/sbin/nologin app
WORKDIR /app

# ── backend ────────────────────────────────────────────────────────────────
FROM runtime-base AS backend
COPY --from=builder /build/target/release/cloud-mailer /usr/local/bin/cloud-mailer
USER app
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=3 \
    CMD curl --fail --silent --show-error http://127.0.0.1:8080/ready || exit 1
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/cloud-mailer"]

# ── frontend ───────────────────────────────────────────────────────────────
FROM runtime-base AS frontend
COPY --from=builder /build/target/release/frontend /usr/local/bin/frontend
COPY --chown=app:app frontend/static /app/frontend/static
USER app
EXPOSE 3000
HEALTHCHECK --interval=15s --timeout=5s --start-period=5s --retries=3 \
    CMD curl --fail --silent --show-error http://127.0.0.1:3000/login || exit 1
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/frontend"]
