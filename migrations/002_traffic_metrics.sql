-- Traffic metrics table for ingress/egress data
CREATE TABLE IF NOT EXISTS bws_traffic_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    direction TEXT NOT NULL CHECK (direction IN ('ingress', 'egress')),
    method TEXT,
    endpoint TEXT NOT NULL,
    status_code INTEGER,
    request_bytes INTEGER DEFAULT 0,
    response_bytes INTEGER DEFAULT 0,
    duration_ms INTEGER,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_traffic_direction ON bws_traffic_metrics(direction);
CREATE INDEX IF NOT EXISTS idx_traffic_time ON bws_traffic_metrics(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_traffic_direction_time ON bws_traffic_metrics(direction, created_at DESC);
