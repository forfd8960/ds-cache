use crate::commands::BasicCommand;

use anyhow::{Result, anyhow};

impl BasicCommand {
    pub fn from_frame_args(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(anyhow!("Empty command".to_string()));
        }

        let cmd_name = args[0].to_uppercase();

        match cmd_name.as_str() {
            "PING" => {
                if args.len() == 1 {
                    Ok(BasicCommand::Ping { message: None })
                } else if args.len() == 2 {
                    Ok(BasicCommand::Ping {
                        message: Some(args[1].clone()),
                    })
                } else {
                    Err(anyhow!(
                        "PING command takes zero or one argument".to_string()
                    ))
                }
            }
            "ECHO" => {
                if args.len() != 2 {
                    return Err(anyhow!(
                        "ECHO command requires exactly one argument".to_string()
                    ));
                }
                Ok(BasicCommand::Echo {
                    message: args[1].clone(),
                })
            }
            "DEL" => {
                if args.len() < 2 {
                    return Err(anyhow!("DEL command requires at least one key".to_string()));
                }
                Ok(BasicCommand::Del {
                    keys: args[1..].to_vec(),
                })
            }
            "EXISTS" => {
                if args.len() < 2 {
                    return Err(anyhow!(
                        "EXISTS command requires at least one key".to_string()
                    ));
                }
                Ok(BasicCommand::Exists {
                    keys: args[1..].to_vec(),
                })
            }
            "EXPIRE" => {
                if args.len() != 3 {
                    return Err(anyhow!(
                        "EXPIRE command requires exactly two arguments".to_string()
                    ));
                }
                let seconds = args[2]
                    .parse::<u64>()
                    .map_err(|_| anyhow!("Invalid seconds value for EXPIRE".to_string()))?;
                Ok(BasicCommand::Expire {
                    key: args[1].clone(),
                    seconds,
                })
            }
            "TTL" => {
                if args.len() != 2 {
                    return Err(anyhow!(
                        "TTL command requires exactly one argument".to_string()
                    ));
                }
                Ok(BasicCommand::TTL {
                    key: args[1].clone(),
                })
            }
            "KEYS" => {
                if args.len() != 2 {
                    return Err(anyhow!(
                        "KEYS command requires exactly one argument".to_string()
                    ));
                }
                Ok(BasicCommand::Keys {
                    pattern: args[1].clone(),
                })
            }
            "TYPE" => {
                if args.len() != 2 {
                    return Err(anyhow!(
                        "TYPE command requires exactly one argument".to_string()
                    ));
                }
                Ok(BasicCommand::Type {
                    key: args[1].clone(),
                })
            }
            _ => Err(anyhow!("Unknown string command: {}", cmd_name)),
        }
    }
}
