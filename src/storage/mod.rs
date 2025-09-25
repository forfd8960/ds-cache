pub mod entry;
pub mod value;

use crate::commands::ZRangeOptions;
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
    Nil,
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

    // ------- Hash Value Methods -------
    pub fn hset(&mut self, key: &str, pairs: Vec<(String, String)>) -> usize {
        let hash_value = match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => {
                match &mut entry.value {
                    Value::Hash(hash) => hash,
                    _ => {
                        // Key exists but is not a hash - overwrite with new hash
                        let new_hash = HashValue {
                            fields: HashMap::new(),
                            encoding: HashEncoding::HashTable,
                        };
                        entry.value = Value::Hash(new_hash);
                        match &mut entry.value {
                            Value::Hash(hash) => hash,
                            _ => unreachable!(),
                        }
                    }
                }
            }
            Some(_) => {
                // Key exists but is expired - remove it and create new hash
                self.data.remove(key);
                let new_hash = HashValue {
                    fields: HashMap::new(),
                    encoding: HashEncoding::HashTable,
                };
                let entry = Entry::new(Value::Hash(new_hash));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::Hash(hash) => hash,
                    _ => unreachable!(),
                }
            }
            None => {
                // Key does not exist - create new hash
                let new_hash = HashValue {
                    fields: HashMap::new(),
                    encoding: HashEncoding::HashTable,
                };

                let entry = Entry::new(Value::Hash(new_hash));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::Hash(hash) => hash,
                    _ => unreachable!(),
                }
            }
        };

        let mut sz = 0 as usize;
        for (field, value) in pairs {
            let key_bs = field.into_bytes();
            let key = &key_bs;

            if !hash_value.fields.contains_key(key) {
                sz += 1;
            }
            hash_value.fields.insert(key_bs, value.into_bytes());
        }

        sz
    }

    pub fn hget(&mut self, key: &str, field: &str) -> Option<Vec<u8>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => hash.fields.get(field.as_bytes()).cloned(),
                _ => None, // Key exists but is not a hash
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn hdel(&mut self, key: &str, fields: &[String]) -> usize {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::Hash(hash) => {
                    let initial_size = hash.fields.len();
                    for field in fields {
                        hash.fields.remove(field.as_bytes());
                    }
                    initial_size - hash.fields.len()
                }
                _ => 0, // Key exists but is not a hash
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                0
            }
            None => 0, // Key does not exist
        }
    }

    pub fn hmset(&mut self, key: &str, pairs: &[(String, String)]) -> usize {
        self.hset(key, pairs.to_vec())
    }

    pub fn hmget(&mut self, key: &str, fields: &[String]) -> Option<Vec<Option<Vec<u8>>>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => {
                    let mut values = Vec::with_capacity(fields.len());
                    for field in fields {
                        values.push(hash.fields.get(field.as_bytes()).cloned());
                    }
                    Some(values)
                }
                _ => None, // Key exists but is not a hash
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn hexists(&mut self, key: &str, field: &str) -> bool {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => hash.fields.contains_key(field.as_bytes()),
                _ => false, // Key exists but is not a hash
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                false
            }
            None => false, // Key does not exist
        }
    }

    pub fn hlen(&mut self, key: &str) -> usize {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => hash.fields.len(),
                _ => 0, // Key exists but is not a hash
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                0
            }
            None => 0, // Key does not exist
        }
    }

    pub fn hkeys(&mut self, key: &str) -> Option<Vec<Vec<u8>>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => Some(hash.fields.keys().cloned().collect()),
                _ => None, // Key exists but is not a hash
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn hvals(&mut self, key: &str) -> Option<Vec<Vec<u8>>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => Some(hash.fields.values().cloned().collect()),
                _ => None, // Key exists but is not a hash
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn hgetall(&mut self, key: &str) -> Option<HashMap<Vec<u8>, Vec<u8>>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => {
                    let mut map = HashMap::new();
                    for (k, v) in &hash.fields {
                        map.insert(k.clone(), v.clone());
                    }
                    Some(map)
                }
                _ => None, // Key exists but is not a hash
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    // -------- Sorted Set Value Methods -------
    pub fn zadd(&mut self, key: &str, members: Vec<(f64, String)>) -> usize {
        let zset_value = match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => {
                match &mut entry.value {
                    Value::SortedSet(zset) => zset,
                    _ => {
                        // Key exists but is not a sorted set - overwrite with new sorted set
                        entry.value = Value::SortedSet(SortedSetValue::new());
                        match &mut entry.value {
                            Value::SortedSet(zset) => zset,
                            _ => unreachable!(),
                        }
                    }
                }
            }
            Some(_) => {
                // Key exists but is expired - remove it and create new sorted set
                self.data.remove(key);
                let entry = Entry::new(Value::SortedSet(SortedSetValue::new()));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::SortedSet(zset) => zset,
                    _ => unreachable!(),
                }
            }
            None => {
                // Key does not exist - create new sorted set
                let entry = Entry::new(Value::SortedSet(SortedSetValue::new()));
                self.data.insert(key.to_string(), entry);
                match &mut self.data.get_mut(key).unwrap().value {
                    Value::SortedSet(zset) => zset,
                    _ => unreachable!(),
                }
            }
        };
        let mut added = 0;
        for (score, member) in members {
            let member_bytes = member.into_bytes();
            let of_score = OrderedFloat::from(score);
            let exists = zset_value.member_scores.contains_key(&member_bytes);
            if !exists {
                zset_value.members.insert(of_score, member_bytes.clone());
                zset_value.member_scores.insert(member_bytes, of_score);
                added += 1;
            }
        }
        added
    }

    pub fn zrem(&mut self, key: &str, members: Vec<String>) -> usize {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::SortedSet(zset) => {
                    let mut removed = 0;
                    for member in members {
                        let member_bytes = member.into_bytes();
                        if let Some(score) = zset.member_scores.remove(&member_bytes) {
                            zset.members.remove(&score);
                            removed += 1;
                        }
                    }
                    removed
                }
                _ => 0, // Key exists but is not a sorted set
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                0
            }
            None => 0, // Key does not exist
        }
    }

    pub fn zrange(
        &mut self,
        key: &str,
        start: i64,
        stop: i64,
        options: ZRangeOptions,
    ) -> Option<Vec<(String, f64)>> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::SortedSet(zset) => {
                    let len = zset.members.len() as i64;

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

                    if start_idx >= stop_idx || start_idx >= zset.members.len() {
                        return Some(vec![]);
                    }

                    let range_iter = zset
                        .members
                        .iter()
                        .skip(start_idx)
                        .take(stop_idx - start_idx);

                    let mut result = Vec::new();
                    for (score, member) in range_iter {
                        if options.with_scores {
                            result.push((String::from_utf8_lossy(member).to_string(), score.0));
                        } else {
                            result.push((String::from_utf8_lossy(member).to_string(), 0.0));
                        }
                    }
                    Some(result)
                }
                _ => None, // Key exists but is not a sorted set
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                None
            }
            None => None, // Key does not exist
        }
    }

    pub fn zcard(&mut self, key: &str) -> usize {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::SortedSet(zset) => zset.members.len(),
                _ => 0, // Key exists but is not a sorted set
            },
            Some(_) => {
                // Key exists but is expired - remove it
                self.data.remove(key);
                0
            }
            None => 0, // Key does not exist
        }
    }

    pub fn zscore(&mut self, key: &str, member: &str) -> Option<f64> {
        match self.data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::SortedSet(zset) => zset.member_scores.get(member.as_bytes()).map(|of| of.0),
                _ => None, // Key exists but is not a sorted set
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
            Value::Nil => "nil",
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
