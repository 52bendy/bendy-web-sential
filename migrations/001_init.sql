-- bendy-web-sential initial schema
-- business prefix: bws_

-- Domains table
CREATE TABLE IF NOT EXISTS bws_domains (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain TEXT NOT NULL UNIQUE,
    description TEXT,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_domains_domain ON bws_domains(domain);
CREATE INDEX IF NOT EXISTS idx_domains_active ON bws_domains(active);

-- Routes table
CREATE TABLE IF NOT EXISTS bws_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    path_pattern TEXT NOT NULL,
    action TEXT NOT NULL DEFAULT 'proxy',
    target TEXT NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (domain_id) REFERENCES bws_domains(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_routes_domain ON bws_routes(domain_id);
CREATE INDEX IF NOT EXISTS idx_routes_active ON bws_routes(active);
CREATE INDEX IF NOT EXISTS idx_routes_priority ON bws_routes(domain_id, priority DESC);

-- Admin users table
CREATE TABLE IF NOT EXISTS bws_admin_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    totp_secret TEXT,
    role TEXT NOT NULL DEFAULT 'admin',
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_admin_users_username ON bws_admin_users(username);

-- Audit log table
CREATE TABLE IF NOT EXISTS bws_audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    username TEXT,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    resource_id INTEGER,
    ip_address TEXT,
    user_agent TEXT,
    details TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES bws_admin_users(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_user ON bws_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON bws_audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_created ON bws_audit_log(created_at DESC);

-- Token blacklist for revocation (Phase 4, but schema here for forward reference)
CREATE TABLE IF NOT EXISTS bws_token_blacklist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token_jti TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    revoked_at TEXT NOT NULL,
    revoked_by INTEGER,
    FOREIGN KEY (revoked_by) REFERENCES bws_admin_users(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_blacklist_jti ON bws_token_blacklist(token_jti);
CREATE INDEX IF NOT EXISTS idx_blacklist_expires ON bws_token_blacklist(expires_at);
