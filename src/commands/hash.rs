use crate::{
    commands::HashCommand,
    protocol::encode::{encode_error, encode_integer, encode_nil, encode_value},
    storage::{
        CacheStore, HashEncoding, HashValue, ListEncoding, ListValue, StringEncoding, StringValue,
        Value,
    },
};
use anyhow::Result;
use redis_protocol::resp2::types::BytesFrame;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct HashHandler {
    pub store: Arc<RwLock<CacheStore>>,
}

impl HashHandler {
    pub fn new(store: Arc<RwLock<CacheStore>>) -> Self {
        Self { store }
    }

    pub async fn handle_cmd(&mut self, cmd: HashCommand) -> Result<BytesFrame> {
        info!("[HashHandler] handle_cmd cmd: {:?}", cmd);

        match cmd {
            HashCommand::HSet { key, pairs } => self.handle_hset(key, pairs).await,
            HashCommand::HGet { key, field } => self.handle_hget(key, field).await,
            HashCommand::HDel { key, fields } => self.handle_hdel(key, fields).await,
            HashCommand::HMSet { key, pairs } => self.handle_hmset(key, pairs).await,
            HashCommand::HMGet { key, fields } => self.handle_hmget(key, fields).await,
            HashCommand::HExists { key, field } => self.handle_hexists(key, field).await,
            HashCommand::HLen { key } => self.handle_hlen(key).await,
            HashCommand::HKeys { key } => self.handle_hkeys(&key).await,
            HashCommand::HVals { key } => self.handle_hvals(&key).await,
            HashCommand::HGetAll { key } => self.handle_hgetall(&key).await,
            _ => encode_error("unknown command"),
        }
    }

    async fn handle_hset(
        &mut self,
        key: String,
        pairs: Vec<(String, String)>,
    ) -> Result<BytesFrame> {
        info!("cmd to hset pairs {:?} to hash: {}", pairs, key);
        let mut store = self.store.write().await;

        let added_count = store.hset(&key, pairs);
        encode_integer(added_count as i64)
    }

    async fn handle_hget(&mut self, key: String, field: String) -> Result<BytesFrame> {
        info!("cmd to hget field {} from hash: {}", field, key);
        let mut store = self.store.write().await;

        if let Some(value) = store.hget(&key, &field) {
            encode_value(Value::String(StringValue {
                data: value,
                encoding: StringEncoding::Raw,
            }))
        } else {
            encode_nil()
        }
    }

    async fn handle_hdel(&mut self, key: String, fields: Vec<String>) -> Result<BytesFrame> {
        info!("cmd to hdel fields {:?} from hash: {}", fields, key);
        let mut store = self.store.write().await;
        let deleted_count = store.hdel(&key, &fields);
        encode_integer(deleted_count as i64)
    }

    async fn handle_hmset(
        &mut self,
        key: String,
        pairs: Vec<(String, String)>,
    ) -> Result<BytesFrame> {
        info!("cmd to hmset pairs {:?} to hash: {}", pairs, key);
        let mut store = self.store.write().await;

        let added_count = store.hmset(&key, &pairs);
        encode_integer(added_count as i64)
    }

    async fn handle_hmget(&mut self, key: String, fields: Vec<String>) -> Result<BytesFrame> {
        info!("cmd to hmget fields {:?} from hash: {}", fields, key);
        let mut store = self.store.write().await;

        let values = store.hmget(&key, &fields);
        match values {
            None => return encode_nil(),
            Some(v) => encode_value(Value::List(ListValue {
                elements: v
                    .into_iter()
                    .map(|v| v.unwrap_or_else(|| b"(nil)".to_vec()))
                    .collect(),
                encoding: ListEncoding::Quicklist,
            })),
        }
    }

    async fn handle_hexists(&mut self, key: String, field: String) -> Result<BytesFrame> {
        info!("cmd to hexists field {} in hash: {}", field, key);
        let mut store = self.store.write().await;
        let exists = store.hexists(&key, &field);
        encode_integer(if exists { 1 } else { 0 })
    }

    async fn handle_hlen(&mut self, key: String) -> Result<BytesFrame> {
        info!("cmd to get length of hash: {}", key);
        let mut store = self.store.write().await;
        let hash_length = store.hlen(&key);
        encode_integer(hash_length as i64)
    }

    async fn handle_hkeys(&mut self, key: &str) -> Result<BytesFrame> {
        info!("cmd to get keys of hash: {}", key);
        let mut store = self.store.write().await;

        if let Some(keys) = store.hkeys(&key) {
            let key_objs: Vec<BytesFrame> = keys
                .into_iter()
                .map(|k| BytesFrame::BulkString(k.into()))
                .collect();
            Ok(BytesFrame::Array(key_objs))
        } else {
            Ok(BytesFrame::Array(vec![]))
        }
    }

    async fn handle_hvals(&mut self, key: &str) -> Result<BytesFrame> {
        info!("cmd to get values of hash: {}", key);
        let mut store = self.store.write().await;

        if let Some(values) = store.hvals(&key) {
            let value_objs: Vec<BytesFrame> = values
                .into_iter()
                .map(|v| BytesFrame::BulkString(v.into()))
                .collect();
            Ok(BytesFrame::Array(value_objs))
        } else {
            Ok(BytesFrame::Array(vec![]))
        }
    }

    async fn handle_hgetall(&mut self, key: &str) -> Result<BytesFrame> {
        info!("cmd to get all key-value pairs of hash: {}", key);
        let mut store = self.store.write().await;

        if let Some(hash) = store.hgetall(&key) {
            let mut arr = Vec::with_capacity(hash.len() * 2);
            for (k, v) in hash {
                arr.push(BytesFrame::BulkString(k.into()));
                arr.push(BytesFrame::BulkString(v.into()));
            }
            Ok(BytesFrame::Array(arr))
        } else {
            Ok(BytesFrame::Array(vec![]))
        }
    }
}
