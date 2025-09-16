use super::{Command, StringCommand};
use crate::{
    protocol::encode::{encode_error, encode_value},
    storage::{CacheStore, StringValue, Value},
};

use anyhow::{Result, anyhow};
use redis_protocol::resp2::types::BytesFrame;

pub struct CmdHandler {}

struct StringHandler {}

impl StringHandler {
    pub fn handle_string_value(cmd: StringCommand, store: &mut CacheStore) -> Result<BytesFrame> {
        println!("[handle_string_value] StringCommand: {:?}", cmd);

        match cmd {
            StringCommand::Get { key } => {
                println!("cmd to get value by: {}", key);
                let value = store.get(&key);
                if value.is_none() {
                    encode_error("key not found")
                } else {
                    encode_value(value.unwrap())
                }
            }
            StringCommand::Set { key, value, .. } => {
                println!("cmd to set value {}: {}", key, value);
                let v = Value::String(StringValue::new(value));
                store.set(key, v.clone());
                encode_value(Value::String(StringValue::new("OK")))
            }
            _ => encode_error("Unknown command"),
        }
    }
}

struct ListHandler {}

impl ListHandler {
    pub fn handle_list_value(cmd: Command, store: &CacheStore) -> Result<BytesFrame> {
        encode_value(Value::String(StringValue::new("OK")))
    }
}

struct SetHandler {}

impl SetHandler {
    pub fn handle_set_value(cmd: Command, store: &CacheStore) -> Result<BytesFrame> {
        encode_value(Value::String(StringValue::new("OK")))
    }
}

struct SortedSetHandler {}

impl SortedSetHandler {
    pub fn handle_sorted_set_value(cmd: Command, store: &CacheStore) -> Result<BytesFrame> {
        encode_value(Value::String(StringValue::new("OK")))
    }
}

struct HashHandler {}

impl HashHandler {
    pub fn handle_hash_value(cmd: Command, store: &CacheStore) -> Result<BytesFrame> {
        encode_value(Value::String(StringValue::new("OK")))
    }
}

impl CmdHandler {
    pub fn handle_cmd(cmd: Command, store: &mut CacheStore) -> Result<BytesFrame> {
        println!("[CmdHandler] handle_cmd cmd: {:?}", cmd);

        match cmd {
            Command::String(s_cmd) => StringHandler::handle_string_value(s_cmd, store),
            _ => Err(anyhow!("unknown command")),
        }
    }
}
