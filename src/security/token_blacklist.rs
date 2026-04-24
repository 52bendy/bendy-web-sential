use std::collections::HashSet;
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct TokenBlacklist {
    inner: RwLock<HashSet<String>>,
    expiry: RwLock<std::collections::HashMap<String, Instant>>,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashSet::new()),
            expiry: RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub fn revoke(&self, jti: &str, ttl: Duration) {
        {
            let mut set = self.inner.write().unwrap();
            set.insert(jti.to_string());
        }
        {
            let mut exp = self.expiry.write().unwrap();
            exp.insert(jti.to_string(), Instant::now() + ttl);
        }
    }

    pub fn is_revoked(&self, jti: &str) -> bool {
        let revoked = {
            let set = self.inner.read().unwrap();
            set.contains(jti)
        };
        if revoked {
            let expired = {
                let exp = self.expiry.read().unwrap();
                exp.get(jti).map(|i| i.elapsed() > Duration::from_secs(0)).unwrap_or(true)
            };
            if expired {
                let _ = expired;
                let mut set = self.inner.write().unwrap();
                set.remove(jti);
                return false;
            }
            return true;
        }
        false
    }

    pub fn cleanup(&self) {
        let now = Instant::now();
        let mut set = self.inner.write().unwrap();
        let mut exp = self.expiry.write().unwrap();
        let expired_keys: Vec<String> = exp.iter()
            .filter(|(_, e)| now >= **e)
            .map(|(k, _)| k.clone())
            .collect();
        for k in &expired_keys {
            set.remove(k);
            exp.remove(k);
        }
    }
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist_revoke_and_check() {
        let blacklist = TokenBlacklist::new();
        let jti = "test-jti-123";
        let ttl = Duration::from_secs(60);

        assert!(!blacklist.is_revoked(jti), "should not be revoked before add");
        blacklist.revoke(jti, ttl);
        assert!(blacklist.is_revoked(jti), "should be revoked after add");
    }

    #[test]
    fn test_blacklist_nonexistent() {
        let blacklist = TokenBlacklist::new();
        assert!(!blacklist.is_revoked("nonexistent-jti"));
    }

    #[test]
    fn test_blacklist_multiple_tokens() {
        let blacklist = TokenBlacklist::new();
        let ttl = Duration::from_secs(60);

        blacklist.revoke("jti-1", ttl);
        blacklist.revoke("jti-2", ttl);
        blacklist.revoke("jti-3", ttl);

        assert!(blacklist.is_revoked("jti-1"));
        assert!(blacklist.is_revoked("jti-2"));
        assert!(blacklist.is_revoked("jti-3"));
        assert!(!blacklist.is_revoked("jti-4"));
    }

    #[test]
    fn test_blacklist_cleanup() {
        let blacklist = TokenBlacklist::new();
        let ttl_short = Duration::from_millis(10);
        let ttl_long = Duration::from_secs(3600);

        blacklist.revoke("short-lived", ttl_short);
        blacklist.revoke("long-lived", ttl_long);

        std::thread::sleep(Duration::from_millis(20));
        blacklist.cleanup();

        assert!(!blacklist.is_revoked("short-lived"), "expired token should be cleaned");
        assert!(blacklist.is_revoked("long-lived"), "valid token should remain");
    }
}
