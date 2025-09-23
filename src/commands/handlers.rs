use super::{Command, StringCommand};
use crate::{
    commands::{list::ListHandler, string::StringHandler},
    protocol::encode::{encode_error, encode_value},
    storage::{CacheStore, StringValue, Value},
};
use std::sync::Arc;

use anyhow::{Result, anyhow};
use redis_protocol::resp2::types::BytesFrame;
use tokio::sync::RwLock;

pub struct CmdHandler {
    pub string_handler: StringHandler,
    pub list_handler: ListHandler,
}

impl CmdHandler {
    pub fn new(store: Arc<RwLock<CacheStore>>) -> Self {
        Self {
            string_handler: StringHandler::new(store.clone()),
            list_handler: ListHandler::new(store.clone()),
        }
    }

    pub async fn handle_cmd(&mut self, cmd: Command) -> Result<BytesFrame> {
        println!("[CmdHandler] handle_cmd cmd: {:?}", cmd);

        match cmd {
            Command::String(s_cmd) => self.string_handler.handle_cmd(s_cmd).await,
            Command::List(l_cmd) => self.list_handler.handle_cmd(l_cmd).await,
            _ => Err(anyhow!("unknown command")),
        }
    }
}
