use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use crate::error::{AppError, Result};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init(db_url: &str) -> Result<DbPool> {
    if let Some(parent) = Path::new(db_url).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::ConfigError(e.to_string()))?;
        }
    }

    let conn = Connection::open(db_url)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bws_schema_migrations (
            id INTEGER PRIMARY KEY,
            version TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    let migrations: [(&str, &str); 4] = [
        ("001_init", include_str!("../../migrations/001_init.sql")),
        ("002_traffic", include_str!("../../migrations/002_traffic_metrics.sql")),
        ("003_avatar", include_str!("../../migrations/003_user_avatar.sql")),
        ("004_email", include_str!("../../migrations/004_email.sql")),
    ];

    for (version, sql) in migrations.iter() {
        let exists: bool = conn.query_row(
            "SELECT 1 FROM bws_schema_migrations WHERE version = ?1",
            [version],
            |_| Ok(true),
        ).unwrap_or(false);

        if !exists {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO bws_schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![version, Utc::now().to_rfc3339()],
            )?;
            tracing::info!("applied migration: {}", version);
        }
    }

    Ok(())
}
