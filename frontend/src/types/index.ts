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
