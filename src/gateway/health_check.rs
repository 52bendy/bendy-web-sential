use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use rusqlite::params;

use crate::db::DbPool;
use crate::types::Upstream;

pub async fn start_health_checker(db: DbPool, interval_secs: u64) {
    let mut ticker = interval(Duration::from_secs(interval_secs));

    loop {
        ticker.tick().await;
        check_all_upstreams(&db).await;
    }
}

async fn check_all_upstreams(db: &DbPool) {
    let upstreams = get_all_upstreams(db).await;

    for upstream in upstreams {
        let is_healthy = check_upstream_health(&upstream).await;
        update_upstream_health(db, upstream.id, is_healthy).await;
    }
}

async fn check_upstream_health(upstream: &Upstream) -> bool {
    let url = format!("{}/health", upstream.target_url.trim_end_matches('/'));

    match reqwest::get(&url).await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

async fn get_all_upstreams(db: &DbPool) -> Vec<Upstream> {
    let conn = match db.lock() { Ok(c) => c, Err(_) => return vec![] };

    let mut stmt = match conn.prepare(
        "SELECT id, route_id, target_url, weight, active, healthy, created_at FROM bws_upstreams WHERE active = 1"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    stmt.query_map([], |row| {
        use chrono::Utc;
        Ok(Upstream {
            id: row.get(0)?,
            route_id: row.get(1)?,
            target_url: row.get(2)?,
            weight: row.get(3)?,
            active: row.get::<_, i32>(4)? != 0,
            healthy: row.get::<_, i32>(5)? != 0,
            created_at: row.get::<_, String>(6)?
                .parse()
                .unwrap_or_else(|_| Utc::now()),
        })
    }).ok()
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

async fn update_upstream_health(db: &DbPool, upstream_id: i64, healthy: bool) {
    let conn = match db.lock() { Ok(c) => c, Err(_) => return };
    let _ = conn.execute(
        "UPDATE bws_upstreams SET healthy = ?1 WHERE id = ?2",
        params![healthy as i32, upstream_id],
    );
}
