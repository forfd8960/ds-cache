use crate::storage::{ListValue, StringValue, Value};

use anyhow::{Result, anyhow};
use redis_protocol::resp2::types::BytesFrame;

pub fn encode_value(value: Value) -> Result<BytesFrame> {
    match value {
        Value::String(v) => encode_string(v),
        Value::List(v) => encode_list(v),
        _ => Err(anyhow!("{:?} not supported", value)),
    }
}

fn encode_string(s_v: StringValue) -> Result<BytesFrame> {
    Ok(BytesFrame::BulkString(s_v.data.into()))
}

fn encode_list(list_v: ListValue) -> Result<BytesFrame> {
    Ok(BytesFrame::Array(
        list_v
            .elements
            .into_iter()
            .map(|e| BytesFrame::BulkString(e.into()))
            .collect(),
    ))
}

pub fn encode_integer(v: i64) -> Result<BytesFrame> {
    Ok(BytesFrame::Integer(v))
}

pub fn encode_error(err_msg: &str) -> Result<BytesFrame> {
    Ok(BytesFrame::Error(err_msg.into()))
}
