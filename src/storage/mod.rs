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

#[derive(Debug, Clone)]
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

    pub fn lpush(&mut self, key: &str, values: Vec<String>) -> usize {
        let list_value = match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => {
                match &mut entry.value {
                    Value::List(list) => list,
                    _ => {
                        // Key exists but is not a list - overwrite with new list
                        let new_list = ListValue {
                            elements: Vec::new(),
                            encoding: ListEncoding::Quicklist,
                        };
                        entry.value = Value::List(new_list);
                        match &mut entry.value {
                            Value::List(list) => list,
                            _ => unreachable!(),
                        }
                    }
                }
            }
            Some(_) => {
                // Key exists but is expired - remove it and create new list
                self.data.remove(key);
                let new_list = ListValue {
                    elements: Vec::new(),
                    encoding: ListEncoding::Quicklist,
                };
                let entry = Entry::new(Value::List(new_list));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::List(list) => list,
                    _ => unreachable!(),
                }
            }
            None => {
                // Key does not exist - create new list
                let new_list = ListValue {
                    elements: Vec::new(),
                    encoding: ListEncoding::Quicklist,
                };
                let entry = Entry::new(Value::List(new_list));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::List(list) => list,
                    _ => unreachable!(),
                }
            }
        };

        // Prepend values to the list
        for value in values.into_iter().rev() {
            list_value.elements.insert(0, value.into_bytes());
        }

        list_value.elements.len()
    }

    pub fn rpush(&mut self, key: &str, values: Vec<String>) -> usize {
        let list_value = match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => {
                match &mut entry.value {
                    Value::List(list) => list,
                    _ => {
                        // Key exists but is not a list - overwrite with new list
                        let new_list = ListValue {
                            elements: Vec::new(),
                            encoding: ListEncoding::Quicklist,
                        };
                        entry.value = Value::List(new_list);
                        match &mut entry.value {
                            Value::List(list) => list,
                            _ => unreachable!(),
                        }
                    }
                }
            }
            Some(_) => {
                // Key exists but is expired - remove it and create new list
                self.data.remove(key);
                let new_list = ListValue {
                    elements: Vec::new(),
                    encoding: ListEncoding::Quicklist,
                };
                let entry = Entry::new(Value::List(new_list));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::List(list) => list,
                    _ => unreachable!(),
                }
            }
            None => {
                // Key does not exist - create new list
                let new_list = ListValue {
                    elements: Vec::new(),
                    encoding: ListEncoding::Quicklist,
                };
                let entry = Entry::new(Value::List(new_list));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::List(list) => list,
                    _ => unreachable!(),
                }
            }
        };

        // Append values to the list
        for value in values {
            list_value.elements.push(value.into_bytes());
        }

        list_value.elements.len()
    }

    pub fn lpop(&mut self, key: &str, count: u64) -> Option<Vec<Vec<u8>>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::List(list) => {
                    let mut popped = Vec::new();
                    for _ in 0..count {
                        if let Some(value) = list.pop_left() {
                            popped.push(value);
                        } else {
                            break;
                        }
                    }

                    if popped.is_empty() {
                        None
                    } else {
                        Some(popped)
                    }
                }
                _ => None, // Key exists but is not a list
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn rpop(&mut self, key: &str, count: u64) -> Option<Vec<Vec<u8>>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::List(list) => {
                    let mut popped = Vec::new();
                    for _ in 0..count {
                        if let Some(value) = list.pop_right() {
                            popped.push(value);
                        } else {
                            break;
                        }
                    }

                    if popped.is_empty() {
                        None
                    } else {
                        Some(popped)
                    }
                }
                _ => None, // Key exists but is not a list
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn llen(&mut self, key: &str) -> Option<usize> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::List(list) => Some(list.len()),
                _ => None, // Key exists but is not a list
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn lrange(&mut self, key: &str, start: i64, stop: i64) -> Option<Vec<Vec<u8>>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::List(list) => {
                    let len = list.len() as i64;

                    let start_idx = if start < 0 {
                        (len + start).max(0)
                    } else {
                        start.min(len)
                    } as usize;

                    let stop_idx = if stop < 0 {
                        (len + stop + 1).max(0)
                    } else {
                        (stop + 1).min(len)
                    } as usize;

                    if start_idx >= stop_idx || start_idx >= list.len() {
                        return Some(vec![]);
                    }

                    Some(list.elements[start_idx..stop_idx].to_vec())
                }
                _ => None, // Key exists but is not a list
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    // ------- Set Value Methods -------
    pub fn sadd(&mut self, key: &str, members: Vec<String>) -> usize {
        let set_value = match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => {
                match &mut entry.value {
                    Value::Set(set) => set,
                    _ => {
                        // Key exists but is not a set - overwrite with new set
                        let new_set = SetValue {
                            members: HashSet::new(),
                            encoding: SetEncoding::HashTable,
                        };
                        entry.value = Value::Set(new_set);
                        match &mut entry.value {
                            Value::Set(set) => set,
                            _ => unreachable!(),
                        }
                    }
                }
            }
            Some(_) => {
                // Key exists but is expired - remove it and create new set
                self.data.remove(key);
                let new_set = SetValue {
                    members: HashSet::new(),
                    encoding: SetEncoding::HashTable,
                };
                let entry = Entry::new(Value::Set(new_set));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::Set(set) => set,
                    _ => unreachable!(),
                }
            }
            None => {
                // Key does not exist - create new set
                let new_set = SetValue {
                    members: HashSet::new(),
                    encoding: SetEncoding::HashTable,
                };
                let entry = Entry::new(Value::Set(new_set));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::Set(set) => set,
                    _ => unreachable!(),
                }
            }
        };

        let initial_size = set_value.members.len();
        for member in members {
            set_value.members.insert(member.into_bytes());
        }

        set_value.members.len() - initial_size
    }

    pub fn srem(&mut self, key: &str, members: Vec<String>) -> usize {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::Set(set) => {
                    let initial_size = set.members.len();
                    for member in members {
                        set.members.remove(&member.into_bytes());
                    }
                    initial_size - set.members.len()
                }
                _ => 0, // Key exists but is not a set
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                0
            }
            None => 0, // Key does not exist
        }
    }

    pub fn smembers(&mut self, key: &str) -> Option<HashSet<Vec<u8>>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Set(set) => Some(set.members.clone()),
                _ => None, // Key exists but is not a set
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn scard(&mut self, key: &str) -> Option<usize> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Set(set) => Some(set.members.len()),
                _ => None, // Key exists but is not a set
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn s_ismember(&mut self, key: &str, member: &str) -> Option<bool> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Set(set) => Some(set.members.contains(&member.as_bytes().to_vec())),
                _ => None, // Key exists but is not a set
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
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
