-- Rewrite rules table for request/response transformation
CREATE TABLE IF NOT EXISTS bws_rewrite_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    rule_type TEXT NOT NULL DEFAULT 'header_replace',
    pattern TEXT NOT NULL,
    replacement TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_rewrite_enabled ON bws_rewrite_rules(enabled);
