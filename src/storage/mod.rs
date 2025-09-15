pub mod entry;
pub mod value;

use crate::storage::entry::Entry;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    time::Duration,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(StringValue),
    List(ListValue),
    Set(SetValue),
    SortedSet(SortedSetValue),
    Hash(HashValue),
}

// ========== String Value ==========
#[derive(Debug, Clone, PartialEq)]
pub struct StringValue {
    pub data: Vec<u8>, // Store as bytes to handle binary data
    pub encoding: StringEncoding,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringEncoding {
    Raw,    // Raw string/binary data
    Int,    // Integer stored as string
    Embstr, // Embedded string (short strings)
}

// ========== List Value ==========
#[derive(Debug, Clone, PartialEq)]
pub struct ListValue {
    pub elements: Vec<Vec<u8>>, // List of byte arrays
    pub encoding: ListEncoding,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListEncoding {
    Ziplist,    // Compressed list for small lists
    LinkedList, // Standard doubly-linked list
    Quicklist,  // Hybrid of ziplist and linkedlist
}

// ========== Set Value ==========
#[derive(Debug, Clone, PartialEq)]
pub struct SetValue {
    pub members: HashSet<Vec<u8>>, // Set of byte arrays
    pub encoding: SetEncoding,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetEncoding {
    HashTable, // Standard hash table
    IntSet,    // Optimized for integer-only sets
}

// ========== Sorted Set Value ==========
#[derive(Debug, Clone, PartialEq)]
pub struct SortedSetValue {
    // BTreeMap maintains sorted order by score
    pub members: BTreeMap<OrderedFloat, Vec<u8>>,
    // Reverse lookup: member -> score
    pub member_scores: HashMap<Vec<u8>, OrderedFloat>,
    pub encoding: SortedSetEncoding,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortedSetEncoding {
    Ziplist,  // Compressed for small sorted sets
    SkipList, // Skip list + hash table for large sets
}

// Wrapper for f64 to make it Ord for BTreeMap
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct OrderedFloat(pub f64);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl From<f64> for OrderedFloat {
    fn from(f: f64) -> Self {
        OrderedFloat(f)
    }
}

// ========== Hash Value ==========
#[derive(Debug, Clone, PartialEq)]
pub struct HashValue {
    pub fields: HashMap<Vec<u8>, Vec<u8>>, // field -> value mapping
    pub encoding: HashEncoding,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HashEncoding {
    Ziplist,   // Compressed for small hashes
    HashTable, // Standard hash table
}

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
