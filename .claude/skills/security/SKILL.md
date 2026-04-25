---
name: security
description: Security implementation for bendy-web-sential. Use when working on authentication, JWT tokens, TOTP 2FA, API keys, or security hardening.
when_to_use: Implementing auth features, token management, 2FA setup, security audits, input validation.
---

# Security Guide

## Authentication Methods

### 1. JWT Authentication
- Login at `POST /api/v1/auth/login`
- Token in `Authorization: Bearer <token>` header
- Token expiry: 86400 seconds (1 day)

### 2. API Key Authentication
- Key in `X-API-Key: <key>` header
- Stored in `bws_api_keys` table
- Supports role-based access: `superadmin > admin > user`

### 3. TOTP 2FA (Two-Factor)
- Uses `totp-rs` crate
- AES-256-CBC encryption for TOTP secrets
- Key stored in `BWS_TOTP_AES_KEY` environment variable

## Security Modules

### src/security/
```
jwt.rs              - JWT generation and verification
totp.rs             - TOTP 2FA implementation
token_blacklist.rs   - Revoked token management
```

### src/middleware/auth.rs
- JWT Bearer token validation
- API Key validation
- Role-based access control (RBAC)

## Key Security Files

### Database Tables
- `bws_admin_users` - Admin accounts with bcrypt hashed passwords
- `bws_api_keys` - API key credentials
- `bws_token_blacklist` - Revoked JWT tokens
- `bws_audit_log` - Audit trail

### Environment Variables
```bash
BWS_JWT_SECRET=<secret>       # JWT signing secret (required)
BWS_JWT_EXPIRY_SECS=86400      # Token expiry in seconds
BWS_TOTP_AES_KEY=<key>         # TOTP encryption key
```

## Security Requirements

### Input Validation
- All user input must be validated
- XSS prevention
- SQL injection prevention

### Secrets Management
- Keys must NOT be committed to git (.env is in .gitignore)
- Use environment variables for sensitive data
- TOTP secrets are AES encrypted at rest

### Token Revocation
- Tokens can be blacklisted via token_blacklist.rs
- Check blacklist before accepting JWT tokens

## Error Codes
| Code | Meaning |
|------|---------|
| 1001 | Token expired or invalid |
| 1002 | Insufficient permissions |
| 1003 | Invalid credentials or parameters |
| 1004 | Authentication required |

## Security Testing
- Core security modules require ≥80% test coverage
- Run: `cargo test`
- Lint: `cargo clippy`
