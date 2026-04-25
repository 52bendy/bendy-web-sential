---
name: database
description: Database operations for bendy-web-sential SQLite backend. Use when working with database migrations, schema changes, queries, or data management.
when_to_use: Creating new tables, adding migrations, querying data, database backups, schema modifications.
---

# Database Guide

## Database
- **Type**: SQLite
- **Location**: `data/bws.db`
- **ORM**: rusqlite (direct SQL)
- **Prefix**: All tables use `bws_` prefix

## Schema Migrations

### Migration Files
Located in `migrations/` folder:
- `001_init.sql` - Core schema (domains, routes, admin users, audit log)
- `002_traffic_metrics.sql` - Traffic statistics
- `003_user_avatar.sql` - User avatars
- `004_email.sql` - Email configuration
- `005_hosting_service.sql` - Hosting service settings
- `006_gateway_auth.sql` - Auth & upstreams

### Run Migrations
Migrations run automatically at startup via `src/db/mod.rs`

### Creating New Migrations
1. Create `migrations/XXX_description.sql`
2. Use sequential numbering
3. Include `CREATE TABLE IF NOT EXISTS` for idempotency

## Database Tables

### bws_domains
Domain configurations
```sql
id, domain, enabled, created_at, updated_at
```

### bws_routes
Route rules for traffic routing
```sql
id, domain_id, path_pattern, action, target, auth_strategy, min_role, ...
```

### bws_admin_users
Admin accounts with bcrypt hashed passwords
```sql
id, username, password_hash, role, totp_secret_encrypted, enabled, ...
```

### bws_api_keys
API key credentials
```sql
id, name, key_hash, role, expires_at, created_at, ...
```

### bws_audit_log
Audit trail for sensitive operations
```sql
id, user_id, action, details, ip_address, created_at
```

### bws_token_blacklist
Revoked JWT tokens
```sql
id, token_jti, expires_at, created_at
```

### bws_upstreams
Upstream targets for load balancing
```sql
id, name, url, weight, enabled, health_check_url, ...
```

### bws_schema_migrations
Migration tracking
```sql
id, version, applied_at
```

## Backup & Restore

### Create Backup
```bash
./scripts/backup.sh
# Saves: data/bws.db + TOTP key
# Location: backups/
```

### Restore
```bash
./scripts/backup.sh --restore backups/bws_db_YYYYMMDD_HHMMSS.sqlite
```

## Direct SQLite Access
```bash
sqlite3 data/bws.db
sqlite> .tables
sqlite> SELECT * FROM bws_domains;
```