-- Performance indexes for better query performance

-- Routes table: composite index for route lookup
CREATE INDEX IF NOT EXISTS idx_routes_path_active_priority
ON bws_routes(path_pattern, active, priority DESC);

-- Traffic metrics: index for time-range queries
CREATE INDEX IF NOT EXISTS idx_traffic_metrics_direction_time
ON bws_traffic_metrics(direction, created_at DESC);

-- Upstreams: index for healthy/active filtering
CREATE INDEX IF NOT EXISTS idx_upstreams_active_healthy
ON bws_upstreams(route_id, active, healthy);

-- API Keys: index for active keys lookup
CREATE INDEX IF NOT EXISTS idx_api_keys_active
ON bws_api_keys(active);

-- Rewrite rules: index for enabled rules
CREATE INDEX IF NOT EXISTS idx_rewrite_rules_enabled
ON bws_rewrite_rules(enabled);