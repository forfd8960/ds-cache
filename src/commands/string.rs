use std::sync::Arc;

use anyhow::{Result, anyhow};
use redis_protocol::resp2::types::BytesFrame;
use tokio::sync::RwLock;

use crate::{
    commands::StringCommand,
    protocol::encode::{encode_error, encode_value},
    storage::{CacheStore, StringValue, Value},
};

pub struct StringHandler {
    pub store: Arc<RwLock<CacheStore>>,
}

impl StringHandler {
    pub fn new(store: Arc<RwLock<CacheStore>>) -> Self {
        Self { store }
    }

    pub async fn handle_cmd(&mut self, cmd: StringCommand) -> Result<BytesFrame> {
        println!("[StringHandler] handle_cmd cmd: {:?}", cmd);

        match cmd {
            StringCommand::Get { key } => self.handle_get(key).await,
            StringCommand::Set { key, value, .. } => self.handle_set(key, value).await,
            _ => Err(anyhow!("unknown command")),
        }
    }

    async fn handle_get(&mut self, key: String) -> Result<BytesFrame> {
        println!("cmd to get value by: {}", key);
        let mut store = self.store.write().await;

        let value = store.get(&key);
        if value.is_none() {
            encode_error("key not found")
        } else {
            encode_value(value.unwrap())
        }
    }

    async fn handle_set(&mut self, key: String, value: String) -> Result<BytesFrame> {
        println!("cmd to set value {}: {}", key, value);
        let v = Value::String(StringValue::new(value));

        let mut store = self.store.write().await;
        store.set(key, v.clone());
        encode_value(Value::String(StringValue::new("OK")))
    }
}
