use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::http::HeaderMap;

#[derive(Debug, Clone)]
pub struct RateLimitPolicy {
    pub name: &'static str,
    pub max_attempts: u32,
    pub window: Duration,
}

#[derive(Debug, Clone)]
pub struct LockoutPolicy {
    pub max_failures: u32,
    pub window: Duration,
    pub lockout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub retry_after_seconds: u64,
}

#[derive(Debug, Default)]
struct RateLimitBucket {
    count: u32,
    window_started_at: Option<Instant>,
}

#[derive(Debug, Default)]
struct FailureBucket {
    count: u32,
    window_started_at: Option<Instant>,
    locked_until: Option<Instant>,
}

#[derive(Debug, Default)]
struct RateLimitStore {
    limits: HashMap<String, RateLimitBucket>,
    failures: HashMap<String, FailureBucket>,
}

#[derive(Debug, Clone, Default)]
pub struct RateLimiter {
    store: Arc<Mutex<RateLimitStore>>,
}

impl RateLimitPolicy {
    pub const fn new(name: &'static str, max_attempts: u32, window: Duration) -> Self {
        Self {
            name,
            max_attempts,
            window,
        }
    }
}

impl LockoutPolicy {
    pub const fn new(max_failures: u32, window: Duration, lockout: Duration) -> Self {
        Self {
            max_failures,
            window,
            lockout,
        }
    }
}

impl RateLimiter {
    pub fn check(&self, policy: RateLimitPolicy, identity: impl AsRef<str>) -> RateLimitDecision {
        let now = Instant::now();
        let key = format!("{}:{}", policy.name, normalize_identity(identity.as_ref()));
        let mut store = self.store.lock().expect("rate-limit store mutex poisoned");
        let bucket = store.limits.entry(key).or_default();

        if bucket
            .window_started_at
            .map(|started_at| now.duration_since(started_at) >= policy.window)
            .unwrap_or(true)
        {
            bucket.window_started_at = Some(now);
            bucket.count = 0;
        }

        if bucket.count >= policy.max_attempts {
            let retry_after = bucket
                .window_started_at
                .map(|started_at| retry_after_seconds(now, started_at + policy.window))
                .unwrap_or_else(|| policy.window.as_secs().max(1));

            return RateLimitDecision {
                allowed: false,
                retry_after_seconds: retry_after,
            };
        }

        bucket.count += 1;
        RateLimitDecision {
            allowed: true,
            retry_after_seconds: 0,
        }
    }

    pub fn lockout_status(
        &self,
        policy: LockoutPolicy,
        identity: impl AsRef<str>,
    ) -> RateLimitDecision {
        let now = Instant::now();
        let key = normalize_identity(identity.as_ref());
        let mut store = self.store.lock().expect("lockout store mutex poisoned");
        let bucket = store.failures.entry(key).or_default();

        if let Some(locked_until) = bucket.locked_until {
            if locked_until > now {
                return RateLimitDecision {
                    allowed: false,
                    retry_after_seconds: retry_after_seconds(now, locked_until),
                };
            }
            bucket.locked_until = None;
            bucket.count = 0;
            bucket.window_started_at = Some(now);
        }

        if bucket
            .window_started_at
            .map(|started_at| now.duration_since(started_at) >= policy.window)
            .unwrap_or(true)
        {
            bucket.count = 0;
            bucket.window_started_at = Some(now);
        }

        RateLimitDecision {
            allowed: true,
            retry_after_seconds: 0,
        }
    }

    pub fn record_failure(
        &self,
        policy: LockoutPolicy,
        identity: impl AsRef<str>,
    ) -> RateLimitDecision {
        let now = Instant::now();
        let key = normalize_identity(identity.as_ref());
        let mut store = self.store.lock().expect("lockout store mutex poisoned");
        let bucket = store.failures.entry(key).or_default();

        if bucket
            .window_started_at
            .map(|started_at| now.duration_since(started_at) >= policy.window)
            .unwrap_or(true)
        {
            bucket.count = 0;
            bucket.window_started_at = Some(now);
            bucket.locked_until = None;
        }

        bucket.count += 1;
        if bucket.count >= policy.max_failures {
            let locked_until = now + policy.lockout;
            bucket.locked_until = Some(locked_until);
            return RateLimitDecision {
                allowed: false,
                retry_after_seconds: retry_after_seconds(now, locked_until),
            };
        }

        RateLimitDecision {
            allowed: true,
            retry_after_seconds: 0,
        }
    }

    pub fn record_success(&self, identity: impl AsRef<str>) {
        let mut store = self.store.lock().expect("lockout store mutex poisoned");
        store
            .failures
            .remove(&normalize_identity(identity.as_ref()));
    }
}

pub fn client_fingerprint(headers: &HeaderMap) -> String {
    let ip = first_header_value(headers, "x-forwarded-for")
        .and_then(|value| value.split(',').next().map(str::trim).map(str::to_string))
        .filter(|value| !value.is_empty())
        .or_else(|| first_header_value(headers, "x-real-ip"))
        .unwrap_or_else(|| "unknown-ip".to_string());
    let user_agent =
        first_header_value(headers, "user-agent").unwrap_or_else(|| "unknown-agent".to_string());

    format!("{ip}|{user_agent}")
}

pub fn rate_limit_identity(headers: &HeaderMap, subject: impl AsRef<str>) -> String {
    format!(
        "{}|{}",
        client_fingerprint(headers),
        normalize_identity(subject.as_ref())
    )
}

fn first_header_value(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn retry_after_seconds(now: Instant, until: Instant) -> u64 {
    until
        .checked_duration_since(now)
        .unwrap_or_else(|| Duration::from_secs(1))
        .as_secs()
        .max(1)
}

fn normalize_identity(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        "anonymous".into()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::http::{HeaderMap, HeaderValue};

    use super::{LockoutPolicy, RateLimitPolicy, RateLimiter, client_fingerprint};

    #[test]
    fn blocks_after_rate_limit_window_is_exhausted() {
        let limiter = RateLimiter::default();
        let policy = RateLimitPolicy::new("login", 2, Duration::from_secs(60));

        assert!(limiter.check(policy.clone(), "user@example.com").allowed);
        assert!(limiter.check(policy.clone(), "user@example.com").allowed);
        let blocked = limiter.check(policy, "user@example.com");

        assert!(!blocked.allowed);
        assert!(blocked.retry_after_seconds > 0);
    }

    #[test]
    fn locks_after_repeated_failures_and_clears_on_success() {
        let limiter = RateLimiter::default();
        let policy = LockoutPolicy::new(2, Duration::from_secs(60), Duration::from_secs(120));

        assert!(
            limiter
                .lockout_status(policy.clone(), "user@example.com")
                .allowed
        );
        assert!(
            limiter
                .record_failure(policy.clone(), "user@example.com")
                .allowed
        );
        assert!(
            !limiter
                .record_failure(policy.clone(), "user@example.com")
                .allowed
        );
        assert!(!limiter.lockout_status(policy, "user@example.com").allowed);

        limiter.record_success("user@example.com");
        assert!(
            limiter
                .lockout_status(
                    LockoutPolicy::new(2, Duration::from_secs(60), Duration::from_secs(120)),
                    "user@example.com"
                )
                .allowed
        );
    }

    #[test]
    fn client_fingerprint_prefers_forwarded_ip() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.10, 10.0.0.1"),
        );
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0"));

        assert_eq!(
            client_fingerprint(&headers),
            "203.0.113.10|Mozilla/5.0".to_string()
        );
    }
}
