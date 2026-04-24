use axum::{
    extract::State,
    routing::get,
    Router, Json,
};
use serde::Serialize;
use crate::db::DbPool;
use crate::types::ApiResponse;
use crate::error::AppError;

pub fn router(db: DbPool) -> Router {
    let state = TrafficState { db };
    Router::new()
        .route("/api/v1/traffic", get(get_traffic))
        .with_state(state)
}

#[derive(Clone)]
pub struct TrafficState {
    pub db: DbPool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficPoint {
    pub time: String,       // ISO 8601
    pub bytes: u64,
    pub requests: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficData {
    pub ingress: Vec<TrafficPoint>,
    pub egress: Vec<TrafficPoint>,
    pub total_ingress_bytes: u64,
    pub total_egress_bytes: u64,
}

/// GET /api/v1/traffic
/// Returns traffic data for the last 24 hours, aggregated by hour
pub async fn get_traffic(
    State(state): State<TrafficState>,
) -> Result<Json<ApiResponse<TrafficData>>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    
    // Calculate time range: last 24 hours
    let now = chrono::Utc::now();
    let twenty_four_hours_ago = now - chrono::Duration::hours(24);
    
    // Query ingress data (requests coming in)
    let mut ingress_stmt = conn.prepare(
        r#"
        SELECT 
            strftime('%Y-%m-%dT%H:00:00Z', created_at) as hour,
            SUM(request_bytes) as total_bytes,
            COUNT(*) as request_count
        FROM bws_traffic_metrics
        WHERE direction = 'ingress' 
          AND created_at >= ?1
        GROUP BY hour
        ORDER BY hour ASC
        "#
    )?;
    
    let ingress_rows = ingress_stmt.query_map([twenty_four_hours_ago.to_rfc3339()], |row| {
        Ok(TrafficPoint {
            time: row.get(0)?,
            bytes: row.get::<_, i64>(1)? as u64,
            requests: row.get::<_, i64>(2)? as u64,
        })
    })?;
    
    let mut ingress_points = Vec::new();
    let mut total_ingress_bytes: u64 = 0;
    for row in ingress_rows {
        if let Ok(point) = row {
            total_ingress_bytes += point.bytes;
            ingress_points.push(point);
        }
    }
    
    // Query egress data (responses going out)
    let mut egress_stmt = conn.prepare(
        r#"
        SELECT 
            strftime('%Y-%m-%dT%H:00:00Z', created_at) as hour,
            SUM(response_bytes) as total_bytes,
            COUNT(*) as request_count
        FROM bws_traffic_metrics
        WHERE direction = 'egress' 
          AND created_at >= ?1
        GROUP BY hour
        ORDER BY hour ASC
        "#
    )?;
    
    let egress_rows = egress_stmt.query_map([twenty_four_hours_ago.to_rfc3339()], |row| {
        Ok(TrafficPoint {
            time: row.get(0)?,
            bytes: row.get::<_, i64>(1)? as u64,
            requests: row.get::<_, i64>(2)? as u64,
        })
    })?;
    
    let mut egress_points = Vec::new();
    let mut total_egress_bytes: u64 = 0;
    for row in egress_rows {
        if let Ok(point) = row {
            total_egress_bytes += point.bytes;
            egress_points.push(point);
        }
    }
    
    // If no data exists, return empty arrays (not an error)
    let traffic = TrafficData {
        ingress: ingress_points,
        egress: egress_points,
        total_ingress_bytes,
        total_egress_bytes,
    };
    
    Ok(Json(ApiResponse::success(traffic)))
}
