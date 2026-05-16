# Central Mailer

A multi-tenant, self-serve email delivery platform. Users register, connect their own
SMTP transports, upload HTML templates with `{{variable}}` tokens, and send mail
through a single API using their own credentials. The platform never owns the sending
domain — it is a pure delivery proxy.

Built in Rust: Axum API + Leptos SSR dashboard + MongoDB + Redis.

## Quick start (Docker)

Two modes — pick one.

### Mode A — hosted DB (MongoDB Atlas + hosted Redis)

Most common for production / VPS deploys.

```bash
cp .env.example .env
# Edit .env and set at least:
#   MONGODB_URI=mongodb+srv://user:pass@cluster.mongodb.net/?retryWrites=true&w=majority
#   REDIS_HOST=<your hosted redis host>
#   REDIS_PORT=<port>
#   REDIS_PASSWORD=<password>
#   ENCRYPTION_KEY=<openssl rand -hex 32>
#   JWT_SECRET=<openssl rand -base64 48>

docker-compose up --build
# or, in Compose V2:
# docker compose up --build
```

The `mongo` and `redis` services are **profile-gated** — they will NOT start in this
mode. Only `backend` and `frontend` come up; they connect out to your hosted services.

### Mode B — fully local (containerised Mongo + Redis)

```bash
cp .env.example .env
# Set only ENCRYPTION_KEY and JWT_SECRET; leave MONGODB_URI and REDIS_* unset
# (they'll default to the local services inside Docker).

docker-compose --profile local up --build
```

This activates the `local` profile, starting `mongo` + `redis` containers alongside
the app.

### URLs

- API:       http://localhost:8080
- Dashboard: http://localhost:3000

### Generate secrets

```bash
openssl rand -hex 32       # ENCRYPTION_KEY (must be exactly 32 bytes)
openssl rand -base64 48    # JWT_SECRET
```

## Quick start (local dev, no Docker)

You'll need running Mongo + Redis instances. Then:

```bash
cargo run -p central-mailer-backend       # API on :8080
cargo run -p central-mailer-frontend      # Dashboard on :3000 (separate terminal)
```

## Project layout

```
.
├── backend/        Axum API server (the actual send service)
├── frontend/       Leptos SSR dashboard (browser UI)
├── shared/         Serde types shared between the two crates
├── Dockerfile      Multi-stage build → 2 final images (backend, frontend)
├── docker-compose.yml
└── Cargo.toml      Workspace root
```

## API surface

All endpoints under `/v1`:

| Group | Endpoint | Auth |
|---|---|---|
| Auth | `POST /v1/auth/signup`, `/auth/signin`, `/auth/apikey/rotate` | public / JWT |
| Transports | `POST/GET/PUT/DELETE /v1/transports[/:name]`, `/v1/transports/:name/verify` | JWT |
| Templates | `POST/GET/PUT/DELETE /v1/templates[/:name]` | JWT |
| **Send** | `POST /v1/send/:username/:transport/:template` | **API key** |
| Logs | `GET /v1/logs?transport=&template=&status=&page=` | JWT |
| Account | `GET /v1/account`, `DELETE /v1/account` | JWT |
| Health | `GET /health`, `GET /ready` | none |

Send example:

```bash
curl -X POST http://localhost:8080/v1/send/alice/gmail/otp \
  -H "Authorization: <your_api_key>" \
  -H "Content-Type: application/json" \
  -d '{"to":"bob@example.com","otp":"847291"}'
```

## Architecture

Full design — data models, encryption, caching, rate limiting, send flow — lives in
the architecture doc (see git history for `ARCHITECTURE.md`).

## Stopping / cleaning

```bash
docker compose down              # stop containers
docker compose down -v           # also wipe mongo + redis volumes
```
