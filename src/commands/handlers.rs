use super::Command;
use crate::{
    commands::{
        basic::BasicCmdHandler, hash::HashHandler, list::ListHandler, set::SetHandler,
        sorted_set::SortedSetHandler, string::StringHandler,
    },
    storage::CacheStore,
};
use std::sync::Arc;

use anyhow::{Result, anyhow};
use redis_protocol::resp2::types::BytesFrame;
use tokio::sync::RwLock;

pub struct CmdHandler {
    pub string_handler: StringHandler,
    pub list_handler: ListHandler,
    pub set_handler: SetHandler,
    pub hash_handler: HashHandler,
    pub sorted_set_handler: SortedSetHandler,
    pub basic_handler: BasicCmdHandler,
}

impl CmdHandler {
    pub fn new(store: Arc<RwLock<CacheStore>>) -> Self {
        Self {
            string_handler: StringHandler::new(store.clone()),
            list_handler: ListHandler::new(store.clone()),
            set_handler: SetHandler::new(store.clone()),
            hash_handler: HashHandler::new(store.clone()),
            sorted_set_handler: SortedSetHandler::new(store.clone()),
            basic_handler: BasicCmdHandler::new(store.clone()),
        }
    }

    pub async fn handle_cmd(&mut self, cmd: Command) -> Result<BytesFrame> {
        println!("[CmdHandler] handle_cmd cmd: {:?}", cmd);

        match cmd {
            Command::String(s_cmd) => self.string_handler.handle_cmd(s_cmd).await,
            Command::List(l_cmd) => self.list_handler.handle_cmd(l_cmd).await,
            Command::Set(set_cmd) => self.set_handler.handle_cmd(set_cmd).await,
            Command::Hash(hash_cmd) => self.hash_handler.handle_cmd(hash_cmd).await,
            Command::SortedSet(ss_cmd) => self.sorted_set_handler.handle_cmd(ss_cmd).await,
            Command::Basic(b_cmd) => self.basic_handler.handle_cmd(b_cmd).await,
            _ => Err(anyhow!("unknown command")),
        }
    }
}
