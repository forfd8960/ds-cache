use std::collections::{BTreeMap, HashMap, HashSet};

use super::{
    HashEncoding, HashValue, ListEncoding, ListValue, OrderedFloat, SetEncoding, SetValue,
    SortedSetEncoding, SortedSetValue, StringEncoding, StringValue, Value,
};

impl Value {
    // Convert to string representation
    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => String::from_utf8(s.data.clone()).ok(),
            _ => None,
        }
    }

    // Get memory usage estimate
    pub fn memory_usage(&self) -> usize {
        match self {
            Value::String(s) => s.data.len() + std::mem::size_of::<StringValue>(),
            Value::List(l) => {
                l.elements.iter().map(|e| e.len()).sum::<usize>() + std::mem::size_of::<ListValue>()
            }
            Value::Set(s) => {
                s.members.iter().map(|m| m.len()).sum::<usize>() + std::mem::size_of::<SetValue>()
            }
            Value::SortedSet(zs) => {
                zs.members.iter().map(|(_, m)| m.len()).sum::<usize>()
                    + std::mem::size_of::<SortedSetValue>()
            }
            Value::Hash(h) => {
                h.fields
                    .iter()
                    .map(|(k, v)| k.len() + v.len())
                    .sum::<usize>()
                    + std::mem::size_of::<HashValue>()
            }
            Value::Nil => 0,
        }
    }

    // Check if value is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Value::String(s) => s.data.is_empty(),
            Value::List(l) => l.elements.is_empty(),
            Value::Set(s) => s.members.is_empty(),
            Value::SortedSet(zs) => zs.members.is_empty(),
            Value::Hash(h) => h.fields.is_empty(),
            Value::Nil => true,
        }
    }
}

impl StringValue {
    pub fn new<T: Into<Vec<u8>>>(data: T) -> Self {
        let data = data.into();
        let encoding = if data.len() <= 39 {
            StringEncoding::Embstr
        } else {
            StringEncoding::Raw
        };

        Self { data, encoding }
    }

    pub fn from_int(value: i64) -> Self {
        Self {
            data: value.to_string().into_bytes(),
            encoding: StringEncoding::Int,
        }
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    pub fn as_int(&self) -> Option<i64> {
        self.as_str().ok()?.parse().ok()
    }

    pub fn as_float(&self) -> Option<f64> {
        self.as_str().ok()?.parse().ok()
    }
}

impl ListValue {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            encoding: ListEncoding::Quicklist,
        }
    }

    pub fn push_left<T: Into<Vec<u8>>>(&mut self, value: T) {
        self.elements.insert(0, value.into());
    }

    pub fn push_right<T: Into<Vec<u8>>>(&mut self, value: T) {
        self.elements.push(value.into());
    }

    pub fn pop_left(&mut self) -> Option<Vec<u8>> {
        if !self.elements.is_empty() {
            Some(self.elements.remove(0))
        } else {
            None
        }
    }

    pub fn pop_right(&mut self) -> Option<Vec<u8>> {
        self.elements.pop()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn get(&self, index: i64) -> Option<&Vec<u8>> {
        let len = self.elements.len() as i64;
        let actual_index = if index < 0 { len + index } else { index };

        if actual_index >= 0 && actual_index < len {
            self.elements.get(actual_index as usize)
        } else {
            None
        }
    }
}

impl SetValue {
    pub fn new() -> Self {
        Self {
            members: HashSet::new(),
            encoding: SetEncoding::HashTable,
        }
    }

    pub fn add<T: Into<Vec<u8>>>(&mut self, member: T) -> bool {
        self.members.insert(member.into())
    }

    pub fn remove(&mut self, member: &[u8]) -> bool {
        self.members.remove(member)
    }

    pub fn contains(&self, member: &[u8]) -> bool {
        self.members.contains(member)
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }
}

impl SortedSetValue {
    pub fn new() -> Self {
        Self {
            members: BTreeMap::new(),
            member_scores: HashMap::new(),
            encoding: SortedSetEncoding::SkipList,
        }
    }

    pub fn add(&mut self, score: f64, member: Vec<u8>) -> bool {
        let ordered_score = OrderedFloat(score);

        // Remove existing member if it exists
        if let Some(old_score) = self.member_scores.remove(&member) {
            self.members.remove(&old_score);
        }

        self.members.insert(ordered_score, member.clone());
        self.member_scores.insert(member, ordered_score);
        true
    }

    pub fn remove(&mut self, member: &[u8]) -> bool {
        if let Some(score) = self.member_scores.remove(member) {
            self.members.remove(&score);
            true
        } else {
            false
        }
    }

    pub fn score(&self, member: &[u8]) -> Option<f64> {
        self.member_scores.get(member).map(|s| s.0)
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }
}

impl HashValue {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            encoding: HashEncoding::HashTable,
        }
    }

    pub fn set<K: Into<Vec<u8>>, V: Into<Vec<u8>>>(&mut self, field: K, value: V) {
        self.fields.insert(field.into(), value.into());
    }

    pub fn get(&self, field: &[u8]) -> Option<&Vec<u8>> {
        self.fields.get(field)
    }

    pub fn remove(&mut self, field: &[u8]) -> bool {
        self.fields.remove(field).is_some()
    }

    pub fn contains_field(&self, field: &[u8]) -> bool {
        self.fields.contains_key(field)
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn keys(&self) -> Vec<&Vec<u8>> {
        self.fields.keys().collect()
    }

    pub fn values(&self) -> Vec<&Vec<u8>> {
        self.fields.values().collect()
    }
}
