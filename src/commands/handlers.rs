use super::Command;
use crate::storage::{StringValue, Value};

use anyhow::Result;

#[derive(Debug)]
pub struct CmdHandler {}

struct StringHandler {}

struct ListHandler {}

struct SetHandler {}

struct SortedSetHandler {}

struct HashHandler {}

impl CmdHandler {
    pub fn handle_cmd(&self, cmd: Command) -> Result<Value> {
        Ok(Value::String(StringValue::new("todo!")))
    }
}
