pub mod value;

use crate::storage::value::{Entry, Value};
use std::{collections::HashMap, time::Duration};

#[derive(Debug)]
pub struct CacheStore {
    data: HashMap<String, Entry>,
}

impl CacheStore {
    pub fn new(cap: usize) -> Self {
        Self {
            data: HashMap::with_capacity(cap),
        }
    }

    // Clean up expired keys
    pub fn cleanup_expired(&mut self) -> u64 {
        let expired_keys: Vec<String> = self
            .data
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        let count = expired_keys.len() as u64;
        for key in expired_keys {
            self.data.remove(&key);
        }

        count
    }

    // Get value and update access time
    pub fn get(&mut self, key: &str) -> Option<Value> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => {
                entry.update_access_time();
                Some(entry.value.clone())
            }
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None,
        }
    }

    // Set value without expiration
    pub fn set(&mut self, key: String, value: Value) {
        let entry = Entry::new(value);
        self.data.insert(key, entry);
    }

    // Set value with expiration
    pub fn set_with_expiration(&mut self, key: String, value: Value, ttl: Duration) {
        let entry = Entry::with_expiration(value, ttl);
        self.data.insert(key, entry);
    }

    // Delete key
    pub fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    // Check if key exists (and is not expired)
    pub fn exists(&mut self, key: &str) -> bool {
        match self.data.get(key) {
            Some(entry) if !entry.is_expired() => true,
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                false
            }
            None => false,
        }
    }

    // Get key type
    pub fn key_type(&mut self, key: &str) -> Option<&'static str> {
        self.get(key).map(|value| match value {
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Set(_) => "set",
            Value::SortedSet(_) => "zset",
            Value::Hash(_) => "hash",
        })
    }

    // Set expiration for existing key
    pub fn expire(&mut self, key: &str, ttl: Duration) -> bool {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => {
                entry.set_expiration(ttl);
                true
            }
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                false
            }
            None => false,
        }
    }

    // Remove expiration from key
    pub fn persist(&mut self, key: &str) -> bool {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => {
                entry.remove_expiration();
                true
            }
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                false
            }
            None => false,
        }
    }

    // Get TTL for key
    pub fn ttl(&mut self, key: &str) -> Option<Duration> {
        match self.data.get(key) {
            Some(entry) if !entry.is_expired() => entry.ttl(),
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None,
        }
    }
}
