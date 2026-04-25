---
name: rust-backend
description: Rust backend development for bendy-web-sential. Use when working on src/ directory, Cargo.toml, Rust features, Axum handlers, middleware, or database code.
when_to_use: Implementing API endpoints, adding middleware, database operations, JWT authentication, TOTP, rate limiting, circuit breaker patterns.
---

# Rust Backend Development Guide

## Project Structure
```
src/
├── main.rs           # Entry point, dual-port server (8080 gateway, 3000 admin)
├── api/               # Admin API handlers
├── config/            # Environment configuration
├── db/                # SQLite migrations
├── error.rs           # Error types with IntoResponse
├── gateway/           # Request router
├── middleware/       # Rate limiting, circuit breaker, auth, retry, validation
├── security/         # JWT, TOTP, token blacklist
└── types.rs          # Shared types & API responses
```

## Tech Stack
- **Web Framework**: Axum + Tower
- **Async Runtime**: Tokio
- **Database**: SQLite (rusqlite)
- **Auth**: JWT + bcrypt
- **Logging**: tracing + JSON
- **HTTP Client**: reqwest

## Database Conventions
- All tables prefixed with `bws_`
- Use migrations in `migrations/` folder
- Business prefix for Redis keys and API endpoints: `bws_`

## API Response Format
```json
{"code": 0, "message": "ok", "data": null}
```

## Error Codes
| Code | Meaning |
|------|---------|
| 1001 | Token expired or invalid |
| 1002 | Insufficient permissions |
| 1003 | Invalid credentials or parameters |
| 1004 | Authentication required |
| 2001 | Rate limit exceeded |
| 2002 | Circuit breaker open |
| 3001 | Resource not found |
| 4001 | Internal server error |

## Key Patterns

### Adding a New API Endpoint
1. Add handler in `src/api/`
2. Register route in `main.rs`
3. Use `types.rs` for request/response types
4. Add error handling with `error.rs`

### Middleware Order
1. Validation
2. Auth (JWT/API Key)
3. Rate Limit
4. Circuit Breaker
5. Retry

## Commands
```bash
cargo build          # Development build
cargo test          # Run tests
cargo fmt          # Format code
cargo clippy        # Lint
cargo run           # Start server
```
