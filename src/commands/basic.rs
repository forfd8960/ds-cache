use anyhow::Result;
use redis_protocol::resp2::types::BytesFrame;
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tracing::info;

use crate::{commands::BasicCommand, protocol::encode::encode_error, storage::CacheStore};

pub struct BasicCmdHandler {
    pub store: Arc<RwLock<CacheStore>>,
}

impl BasicCmdHandler {
    pub fn new(store: Arc<RwLock<CacheStore>>) -> Self {
        Self { store }
    }

    pub async fn handle_cmd(&mut self, cmd: BasicCommand) -> Result<BytesFrame> {
        info!("[BasicCmdHandler] handle_cmd cmd: {:?}", cmd);

        match cmd {
            BasicCommand::Ping { message } => self.handle_ping(message).await,
            BasicCommand::Echo { message } => self.handle_echo(message).await,
            BasicCommand::Del { keys } => self.handle_del(keys).await,
            BasicCommand::Exists { keys } => self.handle_exists(keys).await,
            BasicCommand::Expire { key, seconds } => self.handle_expire(key, seconds).await,
            BasicCommand::TTL { key } => self.handle_ttl(key).await,
            BasicCommand::Keys { pattern } => self.handle_keys(pattern).await,
            BasicCommand::Type { key } => self.handle_type(key).await,
            _ => encode_error("unknown command"),
        }
    }

    async fn handle_ping(&mut self, message: Option<String>) -> Result<BytesFrame> {
        info!("cmd to ping with message: {:?}", message);
        match message {
            Some(msg) => Ok(BytesFrame::SimpleString(msg.into())),
            None => Ok(BytesFrame::SimpleString("PONG".into())),
        }
    }

    async fn handle_echo(&mut self, message: String) -> Result<BytesFrame> {
        info!("cmd to echo message: {}", message);
        Ok(BytesFrame::BulkString(message.into()))
    }

    async fn handle_del(&mut self, keys: Vec<String>) -> Result<BytesFrame> {
        info!("cmd to del keys: {:?}", keys);
        // Placeholder implementation
        let mut store = self.store.write().await;
        let deleted_count = store.delete(keys);
        Ok(BytesFrame::Integer(deleted_count as i64))
    }

    async fn handle_exists(&mut self, keys: Vec<String>) -> Result<BytesFrame> {
        info!("cmd to check existence of keys: {:?}", keys);
        // Placeholder implementation
        let mut store = self.store.write().await;
        let exists_count = store.exists(keys);
        Ok(BytesFrame::Integer(exists_count as i64))
    }

    async fn handle_expire(&mut self, key: String, seconds: u64) -> Result<BytesFrame> {
        info!(
            "cmd to set expire for key: {} with seconds: {}",
            key, seconds
        );
        let mut store = self.store.write().await;
        let result = store.expire(&key, Duration::from_secs(seconds));
        match result {
            true => Ok(BytesFrame::Integer(1)),
            false => Ok(BytesFrame::Integer(0)),
        }
    }

    async fn handle_ttl(&mut self, key: String) -> Result<BytesFrame> {
        info!("cmd to get ttl for key: {}", key);
        let mut store = self.store.write().await;
        let ttl = store.ttl(&key);
        match ttl {
            (d, flag) => {
                match flag {
                    1 => Ok(BytesFrame::Integer(d.as_secs() as i64)),
                    0 => Ok(BytesFrame::Integer(-2)), // key exists but expired
                    -1 => Ok(BytesFrame::Integer(-2)), // key existed but now removed due to expiration
                    -2 => Ok(BytesFrame::Integer(-1)), // key does not exist
                    _ => encode_error("unexpected error in TTL command"),
                }
            }
        }
    }

    async fn handle_keys(&mut self, pattern: String) -> Result<BytesFrame> {
        info!("cmd to get keys with pattern: {}", pattern);
        let mut store = self.store.write().await;
        let keys = store.keys(&pattern);
        let frames: Vec<BytesFrame> = keys
            .into_iter()
            .map(|k| BytesFrame::BulkString(k.into()))
            .collect();
        Ok(BytesFrame::Array(frames))
    }

    async fn handle_type(&mut self, key: String) -> Result<BytesFrame> {
        info!("cmd to get type of key: {}", key);
        let mut store = self.store.write().await;
        let data_type = store.type_of(&key);
        match data_type {
            Some(t) => Ok(BytesFrame::BulkString(t.into())),
            None => Ok(BytesFrame::BulkString("none".into())),
        }
    }
}
