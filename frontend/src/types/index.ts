// API response types matching backend
export interface ApiResponse<T = void> {
  code: number;
  message: string;
  data: T | null;
}

export interface Domain {
  id: number;
  domain: string;
  description: string | null;
  hosting_service: string | null;
  active: boolean;
  created_at: string;
  updated_at: string;
}

export interface Route {
  id: number;
  domain_id: number;
  path_pattern: string;
  action: 'proxy' | 'redirect' | 'static';
  target: string;
  description: string | null;
  priority: number;
  active: boolean;
  created_at: string;
  updated_at: string;
  // Auth fields
  auth_strategy: string;
  min_role: string | null;
  // Rate limit fields
  ratelimit_window: number | null;
  ratelimit_limit: number | null;
  ratelimit_dimension: string;
  // Health check fields
  health_check_path: string | null;
  health_check_interval_secs: number;
  // Transform rules
  transform_rules: string | null;
}

export interface AuditLog {
  id: number;
  user_id: number | null;
  username: string | null;
  action: string;
  resource: string;
  resource_id: number | null;
  ip_address: string | null;
  user_agent: string | null;
  details: string | null;
  created_at: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  expires_in: number;
}

export interface MetricsData {
  total_requests: number;
  active_routes: number;
  domains_count: number;
  circuit_breaker_state: string;
}

// User info (from /api/v1/auth/me)
export interface User {
  id: number;
  username: string;
  avatar?: string;
  role: string;
}

// Traffic data (from /api/v1/traffic)
export interface TrafficPoint {
  time: string;       // ISO timestamp
  bytes: number;
  requests: number;
}
export interface TrafficData {
  ingress: TrafficPoint[];
  egress: TrafficPoint[];
  total_ingress_bytes: number;
  total_egress_bytes: number;
}

// Menu position options
export type MenuPosition = 'top' | 'left' | 'bottom';

// Rewrite rule types
export interface RewriteRule {
  id: number;
  name: string;
  rule_type: 'header_add' | 'header_replace' | 'header_remove';
  pattern: string;      // Header name for add/replace, header name for remove
  replacement: string; // Header value for add/replace, empty for remove
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateRewriteRule {
  name: string;
  rule_type: 'header_add' | 'header_replace' | 'header_remove';
  pattern: string;
  replacement: string;
  enabled?: boolean;
}
