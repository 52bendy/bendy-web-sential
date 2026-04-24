use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::sync::RwLock as StdRwLock;
use std::time::{Duration, Instant};

use tokio::sync::RwLock as TokioRwLock;

use crate::config::CircuitBreakerConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    // Sync RwLock for metrics() — called from sync context
    state: Arc<StdRwLock<CircuitState>>,
    last_failure_time: Arc<StdRwLock<Option<Instant>>>,
    // Async RwLock for state transitions — called from async context
    async_state: Arc<TokioRwLock<CircuitState>>,
    async_last_failure_time: Arc<TokioRwLock<Option<Instant>>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    total_requests: Arc<AtomicU32>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(StdRwLock::new(CircuitState::Closed)),
            last_failure_time: Arc::new(StdRwLock::new(None)),
            async_state: Arc::new(TokioRwLock::new(CircuitState::Closed)),
            async_last_failure_time: Arc::new(TokioRwLock::new(None)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            total_requests: Arc::new(AtomicU32::new(0)),
            config,
        }
    }

    pub async fn state(&self) -> CircuitState {
        *self.async_state.read().await
    }

    pub async fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        match *self.async_state.read().await {
            CircuitState::HalfOpen => {
                let successes = self.success_count.load(Ordering::Relaxed);
                if successes >= self.config.success_threshold {
                    tracing::info!("circuit breaker closed");
                    *self.async_state.write().await = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    pub async fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        *self.async_last_failure_time.write().await = Some(Instant::now());

        let state = *self.async_state.read().await;
        if state == CircuitState::HalfOpen {
            tracing::warn!("circuit breaker reopened after failure in half-open state");
            *self.async_state.write().await = CircuitState::Open;
            self.success_count.store(0, Ordering::Relaxed);
            return;
        }

        if state == CircuitState::Closed {
            let failures = self.failure_count.load(Ordering::Relaxed);
            let total = self.total_requests.load(Ordering::Relaxed);
            if failures >= self.config.failure_threshold && total >= self.config.request_volume_threshold {
                let failure_rate = failures as f64 / total.max(1) as f64;
                if failure_rate >= 0.5 {
                    tracing::warn!(failures, total, "circuit breaker opened");
                    *self.async_state.write().await = CircuitState::Open;
                }
            }
        }
    }

    pub async fn try_acquire(&self) -> bool {
        if !self.config.enabled {
            return true;
        }

        let mut state_guard = self.async_state.write().await;
        match *state_guard {
            CircuitState::Open => {
                let last_failure = *self.async_last_failure_time.read().await;
                drop(state_guard);
                if let Some(last) = last_failure {
                    if last.elapsed() >= Duration::from_secs(self.config.open_timeout_secs) {
                        tracing::info!("circuit breaker entering half-open state");
                        *self.async_state.write().await = CircuitState::HalfOpen;
                        self.failure_count.store(0, Ordering::Relaxed);
                        self.success_count.store(0, Ordering::Relaxed);
                        self.total_requests.store(0, Ordering::Relaxed);
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => {
                let in_flight = self.total_requests.load(Ordering::Relaxed);
                in_flight < self.config.half_open_max_requests
            }
            CircuitState::Closed => true,
        }
    }

    pub async fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            state: *self.async_state.read().await,
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 3,
            success_threshold: 2,
            half_open_max_requests: 3,
            open_timeout_secs: 1,
            request_volume_threshold: 5,
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_closed_by_default() {
        let cb = CircuitBreaker::new(config());
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_trip_on_failures() {
        let cb = CircuitBreaker::new(config());
        for _ in 0..6 {
            cb.record_failure().await;
        }
        assert_eq!(cb.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_after_timeout() {
        let cb = CircuitBreaker::new(config());
        for _ in 0..6 {
            cb.record_failure().await;
        }
        assert_eq!(cb.state().await, CircuitState::Open);
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(cb.try_acquire().await);
        assert_eq!(cb.state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_disabled() {
        let mut cfg = config();
        cfg.enabled = false;
        let cb = CircuitBreaker::new(cfg);
        for _ in 0..10 {
            cb.record_failure().await;
        }
        assert!(cb.try_acquire().await);
    }
}