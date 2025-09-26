use std::sync::Arc;

use anyhow::{Result, anyhow};
use redis_protocol::resp2::types::BytesFrame;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    commands::{SetOptions, StringCommand},
    protocol::encode::{encode_error, encode_value},
    storage::{CacheStore, ListEncoding, ListValue, StringValue, Value},
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
            StringCommand::Set {
                key,
                value,
                options,
            } => self.handle_set(key, value, options).await,
            StringCommand::MSet { pairs } => self.handle_mset(pairs).await,
            StringCommand::MGet { keys } => self.handle_mget(keys).await,
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

    async fn handle_set(
        &mut self,
        key: String,
        value: String,
        options: SetOptions,
    ) -> Result<BytesFrame> {
        println!("cmd to set value {}: {}", key, value);
        let v = Value::String(StringValue::new(value));

        let mut store = self.store.write().await;

        let res = store.set(key, v.clone(), options)?;
        if let Some(old_value) = res {
            encode_value(old_value)
        } else {
            encode_value(Value::String(StringValue::new("OK")))
        }
    }

    async fn handle_mset(&mut self, pairs: Vec<(String, String)>) -> Result<BytesFrame> {
        // MSET is atomic, either all keys are set or none are set.
        // If any key is not a string, the entire operation fails and no keys are set.
        // Returns OK if successful, or an error if any key is not a string.

        // For simplicity, we assume the pairs are valid and all keys are strings.
        // In a real implementation, you would need to check the types of existing keys.

        println!("cmd to mset pairs {:?} to string", pairs);
        let mut store = self.store.write().await;

        for (key, value) in pairs {
            let v = Value::String(StringValue::new(value));
            store.set(key, v, SetOptions::default())?;
        }
        encode_value(Value::String(StringValue::new("OK")))
    }

    async fn handle_mget(&mut self, keys: Vec<String>) -> Result<BytesFrame> {
        info!("cmd to mget keys {:?} from string", keys);

        let mut store = self.store.write().await;

        let values = keys
            .into_iter()
            .map(|key| {
                if let Some(Value::String(s)) = store.get(&key) {
                    Some(s.data.into())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        encode_value(Value::List(ListValue {
            elements: values
                .into_iter()
                .map(|v| v.unwrap_or_else(|| b"(nil)".to_vec()))
                .collect(),
            encoding: ListEncoding::Quicklist,
        }))
    }
}
