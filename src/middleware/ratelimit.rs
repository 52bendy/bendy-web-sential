use std::sync::Arc;
use std::net::IpAddr;
use std::collections::HashMap;
use std::num::NonZeroU32;

use tokio::sync::RwLock;
use governor::{RateLimiter, Quota};

use crate::error::AppError;
use crate::config::RateLimitConfig;

pub type IpRateLimiter = RateLimiter<IpAddr, governor::state::keyed::DashMapStateStore<IpAddr>, governor::clock::QuantaClock>;
pub type GlobalRateLimiter = RateLimiter<(), governor::state::keyed::DashMapStateStore<()>, governor::clock::QuantaClock>;

#[derive(Clone)]
pub struct RateLimiters {
    pub ip_limiter: Arc<IpRateLimiter>,
    pub global_limiter: Arc<GlobalRateLimiter>,
    pub path_limiters: Arc<RwLock<HashMap<String, Arc<IpRateLimiter>>>>,
    config: RateLimitConfig,
}

impl RateLimiters {
    pub fn new(config: &RateLimitConfig) -> Self {
        let ip_limiter = RateLimiter::dashmap(Quota::per_second(
            NonZeroU32::new(config.ip_limit_per_second as u32).unwrap()
        ));
        // For global limiter, use a keyed limiter with a single dummy key
        let global_limiter: GlobalRateLimiter = RateLimiter::dashmap(Quota::per_second(
            NonZeroU32::new(config.global_limit_per_second as u32).unwrap()
        ));
        Self {
            ip_limiter: Arc::new(ip_limiter),
            global_limiter: Arc::new(global_limiter),
            path_limiters: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
        }
    }

    pub async fn get_path_limiter(&self, path: &str) -> Arc<IpRateLimiter> {
        let map = self.path_limiters.read().await;
        if let Some(limiter) = map.get(path) {
            return limiter.clone();
        }
        drop(map);
        let mut map = self.path_limiters.write().await;
        if let Some(limiter) = map.get(path) {
            return limiter.clone();
        }
        let limiter = Arc::new(RateLimiter::dashmap(Quota::per_second(
            NonZeroU32::new(self.config.path_limit_per_second as u32).unwrap()
        )));
        map.insert(path.to_string(), limiter.clone());
        limiter
    }

    pub fn check_ip(&self, ip: &IpAddr) -> bool {
        if !self.config.ip_enabled {
            return true;
        }
        self.ip_limiter.check_key(ip).is_ok()
    }

    pub fn check_global(&self) -> bool {
        if !self.config.global_enabled {
            return true;
        }
        // Check with a single dummy key
        self.global_limiter.check_key(&()).is_ok()
    }

    pub async fn check_path(&self, path: &str, ip: &IpAddr) -> bool {
        if !self.config.path_enabled {
            return true;
        }
        let limiter = self.get_path_limiter(path).await;
        limiter.check_key(ip).is_ok()
    }
}

pub async fn check_rate_limit(
    limiters: &RateLimiters,
    ip: &IpAddr,
    path: &str,
) -> Option<AppError> {
    if !limiters.check_ip(ip) {
        tracing::warn!(ip = %ip, "rate limit exceeded for IP");
        return Some(AppError::RateLimited);
    }

    if !limiters.check_global() {
        tracing::warn!("global rate limit exceeded");
        return Some(AppError::RateLimited);
    }

    if !limiters.check_path(path, ip).await {
        tracing::warn!(ip = %ip, path = %path, "rate limit exceeded for path");
        return Some(AppError::RateLimited);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let config = RateLimitConfig {
            ip_enabled: true,
            ip_limit_per_second: 10,
            global_enabled: true,
            global_limit_per_second: 100,
            path_enabled: false,
            path_limit_per_second: 50,
        };
        let _ = RateLimiters::new(&config);
    }

    #[test]
    fn test_rate_limiter_disabled() {
        let config = RateLimitConfig {
            ip_enabled: false,
            ip_limit_per_second: 1,
            global_enabled: false,
            global_limit_per_second: 1,
            path_enabled: false,
            path_limit_per_second: 1,
        };
        let limiters = RateLimiters::new(&config);
        let ip = IpAddr::from([127, 0, 0, 1]);
        assert!(limiters.check_ip(&ip));
        assert!(limiters.check_global());
    }
}