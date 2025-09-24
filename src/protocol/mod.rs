use crate::commands::{Command, HashCommand, ListCommand, SetCommand, StringCommand};
use anyhow::{Result, anyhow};
use redis_protocol::resp2::types::OwnedFrame as Frame;
use tracing::info;

pub mod encode;
pub mod hash;
pub mod list;
pub mod set;
pub mod strings;

fn extract_command_args(frame: Frame) -> Result<Vec<String>> {
    match frame {
        Frame::Array(data) => {
            let mut args = Vec::new();

            for element in data {
                match element {
                    Frame::BulkString(data) => {
                        let arg = String::from_utf8(data).map_err(|_| {
                            anyhow!("Invalid UTF-8 in command argument".to_string())
                        })?;
                        args.push(arg);
                    }
                    Frame::SimpleString(data) => {
                        let arg = String::from_utf8(data).map_err(|_| {
                            anyhow!("Invalid UTF-8 in command argument".to_string())
                        })?;
                        args.push(arg);
                    }
                    _ => {
                        return Err(anyhow!(
                            "Invalid argument type in command array".to_string(),
                        ));
                    }
                }
            }

            Ok(args)
        }
        _ => Err(anyhow!("Expected array frame for command".to_string())),
    }
}

impl From<Frame> for Command {
    fn from(value: Frame) -> Self {
        from_frame(value).unwrap()
    }
}

pub fn from_frame(frame: Frame) -> Result<Command> {
    let args = extract_command_args(frame)?;

    info!("[from_frame] args: {:?}", args);
    if args.is_empty() {
        return Err(anyhow!("Empty command".to_string()));
    }

    let cmd_name = args[0].to_uppercase();
    info!("[from_frame] cmd_name: {}", cmd_name);

    match cmd_name.as_str() {
        // String commands
        "GET" | "SET" | "GETSET" | "SETNX" | "SETEX" | "MGET" | "MSET" | "MSETNX" | "APPEND"
        | "STRLEN" | "INCR" | "INCRBY" | "INCRBYFLOAT" | "DECR" | "DECRBY" | "GETRANGE"
        | "SETRANGE" => Ok(Command::String(StringCommand::from_frame_args(&args)?)),
        // List commands
        "LPUSH" | "RPUSH" | "LPOP" | "RPOP" | "LLEN" | "LRANGE" => {
            Ok(Command::List(ListCommand::from_frame_args(&args)?))
        }
        // Set commands
        "SADD" | "SREM" | "SMEMBERS" | "SCARD" | "SISMEMBER" => {
            Ok(Command::Set(SetCommand::from_frame_args(&args)?))
        }
        // Hash commands
        "HSET" | "HGET" | "HDEL" | "HGETALL" | "HLEN" | "HMSET" | "HMGET" | "HEXISTS" | "HKEYS"
        | "HVALS" => Ok(Command::Hash(HashCommand::from_frame_args(&args)?)),

        // Unknown command
        _ => Ok(Command::Unknown {
            command: cmd_name,
            args: args[1..].to_vec(),
        }),
    }
}
