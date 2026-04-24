-- Migration 004: Add email field to bws_admin_users
-- Add email column if it doesn't exist (for GitHub OAuth users)

-- For SQLite, we need to recreate the table to add a column with NOT NULL constraint
-- First, create new table with email column
CREATE TABLE IF NOT EXISTS bws_admin_users_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL DEFAULT '',
    avatar TEXT,
    email TEXT,
    totp_secret TEXT,
    role TEXT NOT NULL DEFAULT 'user',
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Copy data from old table
INSERT OR IGNORE INTO bws_admin_users_new (id, username, password_hash, avatar, totp_secret, role, active, created_at, updated_at)
SELECT id, username, password_hash, avatar, totp_secret, role, active, created_at, updated_at FROM bws_admin_users;

-- Drop old table and rename new one
DROP TABLE IF EXISTS bws_admin_users;
ALTER TABLE bws_admin_users_new RENAME TO bws_admin_users;

-- Recreate indexes if they existed
CREATE UNIQUE INDEX IF NOT EXISTS idx_bws_admin_users_username ON bws_admin_users(username);
CREATE INDEX IF NOT EXISTS idx_bws_admin_users_role ON bws_admin_users(role);
