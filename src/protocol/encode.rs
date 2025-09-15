use crate::storage::{StringValue, Value};

use anyhow::{Result, anyhow};
use bytes::BytesMut;
use redis_protocol::resp2::{encode, types::OwnedFrame as Frame};

pub fn encode_value(value: Value) -> Result<Frame> {
    match value {
        Value::String(v) => encode_string(v),
        _ => Err(anyhow!("{:?} not supported", value)),
    }
}

fn encode_string(s_v: StringValue) -> Result<Frame> {
    Ok(Frame::SimpleString(s_v.data))
}

pub fn encode_error(err_msg: String) -> Result<Frame> {
    Ok(Frame::Error(err_msg))
}

fn encode_frame(frame: &Frame) -> Result<BytesMut> {
    let mut buf = BytesMut::new();
    encode::encode(&mut buf, frame, false).map_err(|e| anyhow!("Encode error: {}", e))?;
    Ok(buf)
}
