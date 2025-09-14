use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

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

use std::time::{Duration, Instant};

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
