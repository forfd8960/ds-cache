use crate::storage::{StringValue, Value};

use anyhow::{Result, anyhow};
use redis_protocol::resp2::types::BytesFrame;

pub fn encode_value(value: Value) -> Result<BytesFrame> {
    match value {
        Value::String(v) => encode_string(v),
        _ => Err(anyhow!("{:?} not supported", value)),
    }
}

fn encode_string(s_v: StringValue) -> Result<BytesFrame> {
    Ok(BytesFrame::BulkString(s_v.data.into()))
}

pub fn encode_error(err_msg: &str) -> Result<BytesFrame> {
    Ok(BytesFrame::Error(err_msg.into()))
}
