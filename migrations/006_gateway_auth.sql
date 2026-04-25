-- Migration 006: Gateway Authentication & Authorization
-- Adds auth_strategy, min_role to routes, api_keys table, upstreams table

-- Create API Keys table
CREATE TABLE IF NOT EXISTS bws_api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    expires_at TEXT,
    last_used_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_apikeys_hash ON bws_api_keys(key_hash);

-- Create Upstreams table (for load balancing)
CREATE TABLE IF NOT EXISTS bws_upstreams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_id INTEGER NOT NULL,
    target_url TEXT NOT NULL,
    weight INTEGER NOT NULL DEFAULT 1,
    active INTEGER NOT NULL DEFAULT 1,
    healthy INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    FOREIGN KEY (route_id) REFERENCES bws_routes(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_upstreams_route ON bws_upstreams(route_id);

-- Add auth and routing fields to routes
ALTER TABLE bws_routes ADD COLUMN auth_strategy TEXT DEFAULT 'none';
ALTER TABLE bws_routes ADD COLUMN min_role TEXT DEFAULT NULL;
ALTER TABLE bws_routes ADD COLUMN ratelimit_window INTEGER DEFAULT NULL;
ALTER TABLE bws_routes ADD COLUMN ratelimit_limit INTEGER DEFAULT NULL;
ALTER TABLE bws_routes ADD COLUMN ratelimit_dimension TEXT DEFAULT 'ip';
ALTER TABLE bws_routes ADD COLUMN health_check_path TEXT DEFAULT NULL;
ALTER TABLE bws_routes ADD COLUMN health_check_interval_secs INTEGER DEFAULT 30;
ALTER TABLE bws_routes ADD COLUMN transform_rules TEXT DEFAULT NULL;
