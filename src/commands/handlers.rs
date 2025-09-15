use super::{Command, StringCommand};
use crate::storage::{CacheStore, StringValue, Value, entry::Entry};

use anyhow::{Result, anyhow};

pub struct CmdHandler {}

struct StringHandler {}

impl StringHandler {
    pub fn handle_string_value(cmd: StringCommand, store: &mut CacheStore) -> Result<Entry> {
        match cmd {
            StringCommand::Get { key } => {
                println!("cmd to get value by: {}", key);
                let value = store.get(&key);
                if value.is_none() {
                    Err(anyhow!("value not found"))
                } else {
                    Ok(Entry::new(value.unwrap()))
                }
            }
            _ => Err(anyhow!("Unknown command")),
        }
    }
}

struct ListHandler {}

impl ListHandler {
    pub fn handle_list_value(cmd: Command, store: &CacheStore) -> Result<Entry> {
        Ok(Entry::new(Value::String(StringValue::new("OK"))))
    }
}

struct SetHandler {}

impl SetHandler {
    pub fn handle_set_value(cmd: Command, store: &CacheStore) -> Result<Entry> {
        Ok(Entry::new(Value::String(StringValue::new("OK"))))
    }
}

struct SortedSetHandler {}

impl SortedSetHandler {
    pub fn handle_sorted_set_value(cmd: Command, store: &CacheStore) -> Result<Entry> {
        Ok(Entry::new(Value::String(StringValue::new("OK"))))
    }
}

struct HashHandler {}

impl HashHandler {
    pub fn handle_hash_value(cmd: Command, store: &CacheStore) -> Result<Entry> {
        Ok(Entry::new(Value::String(StringValue::new("OK"))))
    }
}

impl CmdHandler {
    pub fn handle_cmd(cmd: Command, store: &mut CacheStore) -> Result<Entry> {
        match cmd {
            Command::String(s_cmd) => StringHandler::handle_string_value(s_cmd, store),
            _ => Err(anyhow!("unknown command")),
        }
    }
}
