use crate::{
    commands::SetCommand,
    protocol::encode::{encode_error, encode_integer, encode_value},
    storage::{CacheStore, SetEncoding, SetValue, Value},
};
use anyhow::{Result, anyhow};
use redis_protocol::resp2::types::BytesFrame;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct SetHandler {
    pub store: Arc<RwLock<CacheStore>>,
}

impl SetHandler {
    pub fn new(store: Arc<RwLock<CacheStore>>) -> Self {
        Self { store }
    }

    pub async fn handle_cmd(&mut self, cmd: SetCommand) -> Result<BytesFrame> {
        info!("[SetHandler] handle_cmd cmd: {:?}", cmd);

        match cmd {
            SetCommand::SAdd { key, members } => self.handle_sadd(&key, members).await,
            SetCommand::SRem { key, members } => self.handle_srem(&key, members).await,
            SetCommand::SMembers { key } => self.handle_smembers(&key).await,
            SetCommand::SCard { key } => self.handle_scard(&key).await,
            SetCommand::SIsMember { key, member } => self.handle_sismember(&key, &member).await,
            _ => Err(anyhow!("command {:#?} not support yet", cmd)),
        }
    }

    async fn handle_sadd(&mut self, key: &str, members: Vec<String>) -> Result<BytesFrame> {
        info!("cmd to set members {:?} to set", members);

        let mut store = self.store.write().await;
        let count = store.sadd(key, members);
        encode_integer(count as i64)
    }

    async fn handle_srem(&mut self, key: &str, members: Vec<String>) -> Result<BytesFrame> {
        info!("cmd to remove members {:?} from set", members);

        let mut store = self.store.write().await;
        let count = store.srem(key, members);
        encode_integer(count as i64)
    }

    async fn handle_smembers(&mut self, key: &str) -> Result<BytesFrame> {
        info!("cmd to get all members of set: {}", key);

        let mut store = self.store.write().await;
        let members = store.smembers(key);
        if members.is_none() {
            encode_error("key not found or not a set")
        } else {
            encode_value(Value::Set(SetValue {
                members: members.unwrap(),
                encoding: SetEncoding::HashTable,
            }))
        }
    }

    async fn handle_scard(&mut self, key: &str) -> Result<BytesFrame> {
        info!("cmd to get cardinality of set: {}", key);

        let mut store = self.store.write().await;
        let count = store.scard(key);
        if count.is_none() {
            encode_error("key not found or not a set")
        } else {
            encode_integer(count.unwrap() as i64)
        }
    }

    async fn handle_sismember(&mut self, key: &str, member: &str) -> Result<BytesFrame> {
        info!("cmd to check if member {} is in set: {}", member, key);

        let mut store = self.store.write().await;
        let is_member = store.s_ismember(key, member);
        if is_member.is_none() {
            encode_error("key not found or not a set")
        } else {
            encode_integer(if is_member.unwrap() { 1 } else { 0 })
        }
    }
}
