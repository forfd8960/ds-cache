use crate::{
    commands::{SortedSetCommand, ZAddOptions, ZRangeOptions},
    protocol::encode::{encode_error, encode_integer, encode_nil, encode_sorted_set, encode_value},
    storage::{CacheStore, StringEncoding, StringValue, Value},
};
use anyhow::Result;
use bytes::Bytes;
use redis_protocol::resp2::types::BytesFrame;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct SortedSetHandler {
    pub store: Arc<RwLock<CacheStore>>,
}

impl SortedSetHandler {
    pub fn new(store: Arc<RwLock<CacheStore>>) -> Self {
        Self { store }
    }

    pub async fn handle_cmd(&mut self, cmd: SortedSetCommand) -> Result<BytesFrame> {
        info!("[SortedSetHandler] handle_cmd cmd: {:?}", cmd);
        match cmd {
            SortedSetCommand::ZAdd {
                key,
                options,
                members,
            } => self.handle_zadd(key, options, members).await,
            SortedSetCommand::ZRem { key, members } => self.handle_zrem(key, members).await,
            SortedSetCommand::ZCard { key } => self.handle_zcard(key).await,
            SortedSetCommand::ZScore { key, member } => self.handle_zscore(key, member).await,
            SortedSetCommand::ZRange {
                key,
                start,
                stop,
                options,
            } => self.handle_zrange(key, start, stop, options).await,
            _ => encode_error("unknown command"),
        }
    }

    async fn handle_zadd(
        &mut self,
        key: String,
        _: ZAddOptions,
        members: Vec<(f64, String)>,
    ) -> Result<BytesFrame> {
        info!("cmd to zadd members {:?} to sorted set: {}", members, key);
        let mut store = self.store.write().await;

        let added_count = store.zadd(&key, members);
        encode_integer(added_count as i64)
    }

    async fn handle_zrem(&mut self, key: String, members: Vec<String>) -> Result<BytesFrame> {
        info!("cmd to zrem members {:?} from sorted set: {}", members, key);
        let mut store = self.store.write().await;

        let removed_count = store.zrem(&key, members);
        encode_integer(removed_count as i64)
    }

    async fn handle_zcard(&mut self, key: String) -> Result<BytesFrame> {
        info!("cmd to zcard sorted set: {}", key);
        let mut store = self.store.write().await;

        let card = store.zcard(&key);
        encode_integer(card as i64)
    }

    async fn handle_zscore(&mut self, key: String, member: String) -> Result<BytesFrame> {
        info!("cmd to zscore member {} from sorted set: {}", member, key);
        let mut store = self.store.write().await;

        if let Some(score) = store.zscore(&key, &member) {
            encode_value(Value::String(StringValue {
                data: Bytes::from(score.to_string()).to_vec(),
                encoding: StringEncoding::Raw,
            }))
        } else {
            encode_nil()
        }
    }

    async fn handle_zrange(
        &mut self,
        key: String,
        start: i64,
        stop: i64,
        options: ZRangeOptions,
    ) -> Result<BytesFrame> {
        info!(
            "cmd to zrange from sorted set: {}, start: {}, stop: {}",
            key, start, stop
        );
        let mut store = self.store.write().await;

        let members = store.zrange(&key, start, stop, options);
        if members.is_none() {
            encode_nil()
        } else {
            encode_sorted_set(members.unwrap())
        }
    }
}
