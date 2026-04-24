# bendy-web-sential

A Rust-based Web traffic control platform that manages multi-domain routing, rate limiting, circuit breaking, and traffic monitoring.

## Overview

bendy-web-sential acts as an intelligent gateway layer in front of your services. It routes incoming requests based on domain and path patterns to different backends (proxy, redirect, or static files), with built-in support for traffic control, authentication, and observability.

## Tech Stack

| Component | Technology |
|---|---|
| Language | Rust (stable) |
| Web Framework | Axum + Tower |
| Async Runtime | Tokio |
| Database | SQLite (rusqlite) |
| Authentication | JWT + bcrypt |
| Logging | tracing + JSON |
| HTTP Client | reqwest |

## Business Prefix

All database tables, Redis keys, and API endpoints use the `bws_` prefix:

```
bws_domains       — Domain configurations
bws_routes        — Route rules
bws_admin_users   — Admin accounts
bws_audit_log     — Audit trail
bws_schema_migrations — DB migrations
bws_token_blacklist  — Revoked tokens (Phase 4)
```

## Directory Structure

```
bendy-web-sential/
├── src/
│   ├── main.rs              # Entry point, dual-port server
│   ├── api/                  # Admin API handlers
│   │   ├── auth.rs          # Login/logout/me endpoints
│   │   └── domains.rs       # Domain & route CRUD
│   ├── config/
│   │   └── mod.rs           # AppConfig from env vars
│   ├── db/
│   │   └── mod.rs           # SQLite init + migrations
│   ├── error.rs             # Error types + IntoResponse
│   ├── gateway/
│   │   └── proxy.rs         # Gateway request router
│   ├── middleware/
│   │   └── log.rs           # Request logging
│   ├── security/
│   │   └── jwt.rs           # JWT generation/verification
│   └── types.rs             # Shared types & API response
├── migrations/
│   └── 001_init.sql          # Schema migrations
├── data/                     # SQLite database (auto-created)
├── .env                      # Local config (gitignored)
├── .env.example             # Config template
├── Cargo.toml
└── README.md
```

## Quick Start

### Prerequisites

- Rust 1.85+
- SQLite3 development headers

### Setup

```bash
# Clone and enter project
cd bendy-web-sential

# Copy env config
cp .env.example .env

# Build
cargo build --release

# Run
cargo run --release
```

The server starts two ports:
- **Gateway**: `http://localhost:8080` — receives traffic
- **Admin API**: `http://localhost:3000` — management interface

### Default Admin Account

On first start, a default admin user is created:

```
Username: admin
Password: bendy2024
```

**Change this password immediately in production.**

## API Reference

### Authentication

#### POST /api/v1/auth/login
```json
// Request
{"username": "admin", "password": "bendy2024"}

// Response
{"code": 0, "message": "ok", "data": {"token": "eyJ...", "expires_in": 86400}}
```

#### POST /api/v1/auth/logout
```json
// Response
{"code": 0, "message": "ok", "data": null}
```

#### GET /api/v1/auth/me
Returns authenticated user info.

### Domain Management

| Method | Endpoint | Description |
|---|---|---|
| GET | /api/v1/domains | List all domains |
| POST | /api/v1/domains | Create domain |
| GET | /api/v1/domains/:id | Get domain |
| PUT | /api/v1/domains/:id | Update domain |
| DELETE | /api/v1/domains/:id | Delete domain |

### Route Management

| Method | Endpoint | Description |
|---|---|---|
| GET | /api/v1/routes | List all routes |
| POST | /api/v1/routes | Create route |
| PUT | /api/v1/routes/:id | Update route |
| DELETE | /api/v1/routes/:id | Delete route |

## Route Actions

| Action | Behavior |
|---|---|
| `proxy` | Forward request to `target` URL |
| `redirect` | HTTP 302 redirect to `target` URL |
| `static` | Serve file from `target` filesystem path |

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `BWS_GATEWAY_PORT` | 8080 | Gateway listening port |
| `BWS_ADMIN_PORT` | 3000 | Admin API listening port |
| `BWS_DATABASE_URL` | data/bws.db | SQLite database path |
| `BWS_JWT_SECRET` | changeme... | JWT signing secret |
| `BWS_JWT_EXPIRY_SECS` | 86400 | Token expiry (seconds) |
| `BWS_LOG_LEVEL` | info | Log level (trace/debug/info/warn/error) |

## Error Codes

| Code | Meaning |
|---|---|
| 1001 | Token expired or invalid |
| 1002 | Insufficient permissions |
| 1003 | Invalid credentials or parameters |
| 1004 | Authentication required |
| 2001 | Rate limit exceeded |
| 2002 | Circuit breaker open |
| 3001 | Resource not found |
| 4001 | Internal server error |
| 4002 | Database error |
| 4003 | Configuration error |
| 4004 | Upstream error |

All errors return `{"code": N, "message": "...", "data": null}`.

## Version

Current: **v0.2.0** (Phase 1 — Core Foundation)

See [maintain.md](maintain.md) for version history and [plan.md](plan.md) for development roadmap.

## Development

```bash
# Development build
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Lint
cargo clippy
```

### Branch Strategy
- `main` — production-ready code
- `dev` — integration branch
- `feat/<name>` — feature branches

### Commit Convention
- `feat:` new features
- `fix:` bug fixes
- `chore:` maintenance, deps, releases
- `test:` test code
- `style:` formatting changes
