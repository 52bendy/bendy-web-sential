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
bws_token_blacklist  — Revoked tokens
bws_api_keys      — API Key credentials
bws_upstreams     — Upstream targets (load balancing)
```

## Directory Structure

```
bendy-web-sential/
├── src/
│   ├── main.rs              # Entry point, dual-port server
│   ├── api/                  # Admin API handlers
│   │   ├── auth.rs          # Login/logout/me endpoints
│   │   ├── domains.rs       # Domain & route CRUD
│   │   ├── keys.rs          # API Key management
│   │   ├── metrics.rs       # System metrics
│   │   ├── audit.rs        # Audit log
│   │   ├── traffic.rs       # Traffic API
│   │   ├── k8s.rs           # Kubernetes integration
│   │   └── prometheus.rs   # Prometheus exporter
│   ├── config/
│   │   └── mod.rs           # AppConfig from env vars
│   ├── db/
│   │   └── mod.rs           # SQLite init + migrations
│   ├── error.rs             # Error types + IntoResponse
│   ├── gateway/
│   │   └── proxy.rs         # Gateway request router
│   ├── middleware/
│   │   ├── ratelimit.rs     # Rate limiting
│   │   ├── circuit_breaker.rs  # Circuit breaker
│   │   ├── auth.rs          # JWT & API Key authentication
│   │   ├── retry.rs         # Retry logic
│   │   └── validation.rs    # Request validation
│   ├── security/
│   │   ├── jwt.rs           # JWT generation/verification
│   │   ├── totp.rs          # TOTP 2FA support
│   │   └── token_blacklist.rs # Token revocation
│   └── types.rs             # Shared types & API response
├── frontend/                 # React + Vite admin UI
│   ├── src/
│   │   ├── pages/          # Page components
│   │   ├── components/     # Reusable components
│   │   └── lib/api.ts      # API client
│   └── dist/                # Built frontend
├── scripts/
│   ├── backup.sh           # Backup/restore
│   ├── ci-build.sh         # CI build pipeline
│   └── release.sh          # Release automation
├── migrations/
│   ├── 001_init.sql        # Core schema
│   ├── 002_traffic_metrics.sql
│   ├── 003_user_avatar.sql
│   ├── 004_email.sql
│   ├── 005_hosting_service.sql
│   └── 006_gateway_auth.sql  # Auth & upstreams
├── docs/
│   └── iterations/         # Iteration development records
│       └── README.md
├── data/                    # SQLite database (auto-created)
├── Dockerfile              # Container image
├── docker-compose.yml       # Container orchestration
├── .env.development        # Dev config
├── .env.staging            # Staging config
├── .env.production         # Production config
├── Cargo.toml
└── README.md
```

## Quick Start

### Prerequisites

- Rust 1.95+ (edition2024 support)
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

### API Key Management

| Method | Endpoint | Description |
|---|---|---|
| GET | /api/v1/keys | List all API Keys (key value hidden) |
| POST | /api/v1/keys | Create new API Key |
| DELETE | /api/v1/keys/:id | Revoke API Key |

#### POST /api/v1/keys
```json
// Request
{"name": "production-key", "role": "user", "expires_at": "2025-12-31T23:59:59Z"}

// Response
{"code": 0, "message": "ok", "data": {
  "id": 1,
  "name": "production-key",
  "key": "a1b2c3d4...",  // only returned on creation
  "role": "user",
  "created_at": "2026-04-24T00:00:00Z",
  "expires_at": "2025-12-31T23:59:59Z"
}}
```

### Route Authentication

Routes support three authentication strategies configured via the `auth_strategy` field:

| Strategy | Header Required | Description |
|---|---|---|
| `none` | None | No authentication |
| `jwt` | `Authorization: Bearer <token>` | Validate JWT token |
| `api_key` | `X-API-Key: <key>` | Validate API Key against `bws_api_keys` |

Role-based access control uses `min_role` field with hierarchy: `superadmin > admin > user`.

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
| `BWS_TOTP_AES_KEY` | — | TOTP AES-256-CBC encryption key |
| `BWS_LOG_LEVEL` | info | Log level (trace/debug/info/warn/error) |
| `BWS_RATE_LIMIT_IP_ENABLED` | true | Per-IP rate limiting |
| `BWS_RATE_LIMIT_IP_PER_SECOND` | 10 | Per-IP request limit |
| `BWS_RATE_LIMIT_GLOBAL_ENABLED` | true | Global rate limiting |
| `BWS_RATE_LIMIT_GLOBAL_PER_SECOND` | 1000 | Global request limit |
| `BWS_CIRCUIT_BREAKER_ENABLED` | false | Circuit breaker |
| `BWS_CIRCUIT_BREAKER_FAILURE_THRESHOLD` | 5 | Failure threshold |
| `BWS_CIRCUIT_BREAKER_SUCCESS_THRESHOLD` | 3 | Recovery threshold |
| `BWS_CIRCUIT_BREAKER_OPEN_TIMEOUT_SECS` | 30 | Open state timeout |
| `BWS_RETRY_ENABLED` | false | Retry middleware |
| `BWS_RETRY_MAX_ATTEMPTS` | 3 | Max retry attempts |
| `BWS_RETRY_BASE_DELAY_MS` | 100 | Retry base delay (ms) |
| `BWS_RETRY_MAX_DELAY_MS` | 5000 | Retry max delay (ms) |
| `BWS_BACKUP_ENABLED` | false | Enable backup |
| `BWS_BACKUP_DIR` | backups | Backup directory |
| `BWS_BACKUP_INTERVAL_HOURS` | 24 | Backup interval |
| `BWS_BACKUP_RETENTION_DAYS` | 7 | Backup retention |

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

## Docker Deployment

```bash
# Generate secrets
export BWS_JWT_SECRET=$(openssl rand -base64 32)
export BWS_TOTP_AES_KEY=$(openssl rand -base64 32)

# Start services
docker-compose up -d

# Check health
docker-compose ps
docker-compose logs --tail=50
```

The gateway will be available at `http://localhost:8080`
The admin API will be available at `http://localhost:8081`

## Backup and Restore

```bash
# Create backup (saves DB + TOTP key)
./scripts/backup.sh

# List backups
./scripts/backup.sh --list

# Restore database
./scripts/backup.sh --restore backups/bws_db_20240115_120000.sqlite
```

## Version

Current: **v0.1.1** (Gateway Authentication & Authorization)

See [docs/iterations/README.md](docs/iterations/README.md) for iteration history and [plan.md](plan.md) for development roadmap.

### Development Iterations

| Version | Feature | Status |
|--------|---------|--------|
| [0.1.1](docs/iterations/iter-0.1.1-auth.md) | 认证与鉴权 (JWT / API Key / 角色权限) | Completed |
| 0.1.2 | 路由级限流 | Pending |
| 0.1.3 | 负载均衡 + 健康检查 | Pending |
| 0.1.4 | 请求/响应改写 | Pending |
| 0.1.5 | 可观测性 | Pending |
| 0.2.0 | 协议扩展 (WebSocket / gRPC) | Pending |

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
