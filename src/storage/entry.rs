use std::time::{Duration, Instant};

use crate::storage::Value;

#[derive(Debug, Clone)]
pub struct Entry {
    pub value: Value,
    pub expires_at: Option<Instant>,
    pub created_at: Instant,
    pub last_accessed: Option<Instant>,
}

impl Entry {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            expires_at: None,
            created_at: Instant::now(),
            last_accessed: None,
        }
    }

    pub fn with_expiration(value: Value, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Some(Instant::now() + ttl),
            created_at: Instant::now(),
            last_accessed: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Instant::now() > expires_at,
            None => false,
        }
    }

    pub fn update_access_time(&mut self) {
        self.last_accessed = Some(Instant::now());
    }

    pub fn set_expiration(&mut self, ttl: Duration) {
        self.expires_at = Some(Instant::now() + ttl);
    }

    pub fn remove_expiration(&mut self) {
        self.expires_at = None;
    }

    pub fn ttl(&self) -> Option<Duration> {
        match self.expires_at {
            Some(expires_at) => {
                let now = Instant::now();
                if expires_at > now {
                    Some(expires_at - now)
                } else {
                    Some(Duration::ZERO) // Expired
                }
            }
            None => None, // No expiration
        }
    }
}
