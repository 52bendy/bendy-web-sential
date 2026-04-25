use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct TimedCache<T> {
    data: RwLock<Vec<T>>,
    loaded_at: RwLock<Instant>,
    ttl_secs: u64,
}

impl<T: Clone> TimedCache<T> {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            data: RwLock::new(Vec::new()),
            loaded_at: RwLock::new(Instant::now()),
            ttl_secs,
        }
    }

    pub fn get(&self) -> Option<Vec<T>> {
        let elapsed = self.loaded_at.read().unwrap().elapsed().as_secs();
        if elapsed > self.ttl_secs {
            None
        } else {
            Some(self.data.read().unwrap().clone())
        }
    }

    pub fn update(&self, data: Vec<T>) {
        *self.data.write().unwrap() = data;
        *self.loaded_at.write().unwrap() = Instant::now();
    }

    pub fn invalidate(&self) {
        *self.loaded_at.write().unwrap() = Instant::now() - Duration::from_secs(self.ttl_secs + 1);
    }
}

pub type RewriteRulesCache = TimedCache<(String, String, String)>;
pub type UpstreamCache = TimedCache<super::super::types::Upstream>;

#[derive(Clone)]
pub struct Caches {
    pub rewrite_rules: Arc<RewriteRulesCache>,
    pub upstreams: Arc<UpstreamCache>,
}

impl Caches {
    pub fn new() -> Self {
        Self {
            rewrite_rules: Arc::new(RewriteRulesCache::new(300)), // 5 min TTL
            upstreams: Arc::new(UpstreamCache::new(30)),          // 30 sec TTL
        }
    }
}

impl Default for Caches {
    fn default() -> Self {
        Self::new()
    }
}
