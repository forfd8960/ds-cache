use crate::storage::Value;

use anyhow::Result;
use bytes::BytesMut;

pub fn encode_value(value: Value) -> Result<BytesMut> {
    Ok(BytesMut::new())
}
