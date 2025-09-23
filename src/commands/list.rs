use crate::{
    commands::ListCommand,
    protocol::encode::{encode_error, encode_integer, encode_value},
    storage::{CacheStore, ListEncoding, ListValue, Value},
};
use anyhow::Result;
use redis_protocol::resp2::types::BytesFrame;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ListHandler {
    pub store: Arc<RwLock<CacheStore>>,
}

impl ListHandler {
    pub fn new(store: Arc<RwLock<CacheStore>>) -> Self {
        Self { store }
    }

    pub async fn handle_cmd(&mut self, cmd: ListCommand) -> Result<BytesFrame> {
        println!("[ListHandler] handle_cmd cmd: {:?}", cmd);

        match cmd {
            ListCommand::LPush { key, values } => self.handle_lpush(key, values).await,
            ListCommand::RPush { key, values } => self.handle_rpush(key, values).await,
            ListCommand::LPop { key, count } => self.handle_lpop(key, count).await,
            ListCommand::RPop { key, count } => self.handle_rpop(key, count).await,
            ListCommand::LLen { key } => self.handle_llen(key).await,
            ListCommand::LRange { key, start, stop } => self.handle_lrange(key, start, stop).await,
            _ => encode_error("unknown command"),
        }
    }

    async fn handle_lpush(&mut self, key: String, values: Vec<String>) -> Result<BytesFrame> {
        println!("cmd to lpush values {:?} to list: {}", values, key);
        let mut store = self.store.write().await;

        let list_size = store.lpush(&key, values);
        encode_integer(list_size as i64)
    }

    async fn handle_rpush(&mut self, key: String, values: Vec<String>) -> Result<BytesFrame> {
        println!("cmd to rpush values {:?} to list: {}", values, key);
        let mut store = self.store.write().await;

        let list_size = store.rpush(&key, values);
        encode_integer(list_size as i64)
    }

    async fn handle_lpop(&mut self, key: String, count: Option<u64>) -> Result<BytesFrame> {
        println!("cmd to lpop from list: {}, count: {:?}", key, count);
        let mut store = self.store.write().await;

        let popped_values = store.lpop(&key, count.unwrap_or(1));
        if popped_values.is_none() {
            encode_error("key not found or list is empty")
        } else {
            encode_value(Value::List(ListValue {
                elements: popped_values.unwrap(),
                encoding: ListEncoding::Quicklist,
            }))
        }
    }

    async fn handle_rpop(&mut self, key: String, count: Option<u64>) -> Result<BytesFrame> {
        println!("cmd to rpop from list: {}, count: {:?}", key, count);
        let mut store = self.store.write().await;

        let popped_values = store.rpop(&key, count.unwrap_or(1));
        if popped_values.is_none() {
            encode_error("key not found or list is empty")
        } else {
            encode_value(Value::List(ListValue {
                elements: popped_values.unwrap(),
                encoding: ListEncoding::Quicklist,
            }))
        }
    }

    async fn handle_llen(&mut self, key: String) -> Result<BytesFrame> {
        println!("cmd to get length of list: {}", key);
        let mut store = self.store.write().await;

        let list_length = store.llen(&key);
        if list_length.is_none() {
            encode_integer(0 as i64)
        } else {
            encode_integer(list_length.unwrap() as i64)
        }
    }

    async fn handle_lrange(&mut self, key: String, start: i64, stop: i64) -> Result<BytesFrame> {
        println!(
            "cmd to lrange from list: {}, start: {}, stop: {}",
            key, start, stop
        );
        let mut store = self.store.write().await;

        let range_values = store.lrange(&key, start, stop);
        if range_values.is_none() {
            encode_error("key not found or list is empty")
        } else {
            encode_value(Value::List(ListValue {
                elements: range_values.unwrap(),
                encoding: ListEncoding::Quicklist,
            }))
        }
    }
}
