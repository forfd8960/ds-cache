use crate::commands::{SetCondition, SetExpire, SetOptions, StringCommand};
use anyhow::{Result, anyhow};

impl StringCommand {
    pub fn from_frame_args(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(anyhow!("Empty command".to_string()));
        }

        let cmd_name = args[0].to_uppercase();

        match cmd_name.as_str() {
            "GET" => parse_get(args),
            "SET" => parse_set(args),
            "GETSET" => parse_getset(args),
            "SETNX" => parse_setnx(args),
            "SETEX" => parse_setex(args),
            "MGET" => parse_mget(args),
            "MSET" => parse_mset(args),
            "MSETNX" => parse_msetnx(args),
            "APPEND" => parse_append(args),
            "STRLEN" => parse_strlen(args),
            "INCR" => parse_incr(args),
            "INCRBY" => parse_incrby(args),
            "INCRBYFLOAT" => parse_incrbyfloat(args),
            "DECR" => parse_decr(args),
            "DECRBY" => parse_decrby(args),
            "GETRANGE" => parse_getrange(args),
            "SETRANGE" => parse_setrange(args),
            _ => Err(anyhow!("Unknown string command: {}", cmd_name)),
        }
    }
}

// Individual command parsers
fn parse_get(args: &[String]) -> Result<StringCommand> {
    if args.len() != 2 {
        return Err(anyhow!("GET requires exactly 1 argument".to_string()));
    }

    Ok(StringCommand::Get {
        key: args[1].clone(),
    })
}

fn parse_set(args: &[String]) -> Result<StringCommand> {
    if args.len() < 3 {
        return Err(anyhow!("SET requires at least 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let value = args[2].clone();
    let mut options = SetOptions::default();

    // Parse optional SET arguments
    let mut i = 3;
    while i < args.len() {
        match args[i].to_uppercase().as_str() {
            "EX" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("EX requires a value".to_string()));
                }
                let seconds = args[i + 1]
                    .parse::<u64>()
                    .map_err(|_| anyhow!("Invalid EX value".to_string()))?;
                options.expire = Some(SetExpire::Ex(seconds));
                i += 2;
            }
            "PX" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("PX requires a value".to_string()));
                }
                let milliseconds = args[i + 1]
                    .parse::<u64>()
                    .map_err(|_| anyhow!("Invalid PX value".to_string()))?;
                options.expire = Some(SetExpire::Px(milliseconds));
                i += 2;
            }
            "EXAT" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("EXAT requires a value".to_string()));
                }
                let timestamp = args[i + 1]
                    .parse::<u64>()
                    .map_err(|_| anyhow!("Invalid EXAT value".to_string()))?;
                options.expire = Some(SetExpire::ExAt(timestamp));
                i += 2;
            }
            "PXAT" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("PXAT requires a value".to_string()));
                }
                let timestamp = args[i + 1]
                    .parse::<u64>()
                    .map_err(|_| anyhow!("Invalid PXAT value".to_string()))?;
                options.expire = Some(SetExpire::PxAt(timestamp));
                i += 2;
            }
            "NX" => {
                options.condition = Some(SetCondition::Nx);
                i += 1;
            }
            "XX" => {
                options.condition = Some(SetCondition::Xx);
                i += 1;
            }
            "KEEPTTL" => {
                options.expire = Some(SetExpire::KeepTtl);
                i += 1;
            }
            "GET" => {
                options.get = true;
                i += 1;
            }
            _ => {
                return Err(anyhow!("Unknown SET option: {}", args[i]));
            }
        }
    }

    Ok(StringCommand::Set {
        key,
        value,
        options,
    })
}

fn parse_getset(args: &[String]) -> Result<StringCommand> {
    if args.len() != 3 {
        return Err(anyhow!("GETSET requires exactly 2 arguments".to_string(),));
    }

    Ok(StringCommand::GetSet {
        key: args[1].clone(),
        value: args[2].clone(),
    })
}

fn parse_setnx(args: &[String]) -> Result<StringCommand> {
    if args.len() != 3 {
        return Err(anyhow!("SETNX requires exactly 2 arguments".to_string()));
    }

    Ok(StringCommand::SetNx {
        key: args[1].clone(),
        value: args[2].clone(),
    })
}

fn parse_setex(args: &[String]) -> Result<StringCommand> {
    if args.len() != 4 {
        return Err(anyhow!("SETEX requires exactly 3 arguments".to_string()));
    }

    let seconds = args[2]
        .parse::<u64>()
        .map_err(|_| anyhow!("Invalid seconds value for SETEX".to_string()))?;

    Ok(StringCommand::SetEx {
        key: args[1].clone(),
        seconds,
        value: args[3].clone(),
    })
}

fn parse_mget(args: &[String]) -> Result<StringCommand> {
    if args.len() < 2 {
        return Err(anyhow!("MGET requires at least 1 key".to_string()));
    }

    Ok(StringCommand::MGet {
        keys: args[1..].to_vec(),
    })
}

fn parse_mset(args: &[String]) -> Result<StringCommand> {
    if args.len() < 3 || (args.len() - 1) % 2 != 0 {
        return Err(anyhow!(
            "MSET requires an even number of arguments (key-value pairs)".to_string(),
        ));
    }

    let mut pairs = Vec::new();
    let mut i = 1;
    while i < args.len() {
        pairs.push((args[i].clone(), args[i + 1].clone()));
        i += 2;
    }

    Ok(StringCommand::MSet { pairs })
}

fn parse_msetnx(args: &[String]) -> Result<StringCommand> {
    if args.len() < 3 || (args.len() - 1) % 2 != 0 {
        return Err(anyhow!(
            "MSETNX requires an even number of arguments (key-value pairs)".to_string(),
        ));
    }

    let mut pairs = Vec::new();
    let mut i = 1;
    while i < args.len() {
        pairs.push((args[i].clone(), args[i + 1].clone()));
        i += 2;
    }

    Ok(StringCommand::MSetNx { pairs })
}

fn parse_append(args: &[String]) -> Result<StringCommand> {
    if args.len() != 3 {
        return Err(anyhow!("APPEND requires exactly 2 arguments".to_string(),));
    }

    Ok(StringCommand::Append {
        key: args[1].clone(),
        value: args[2].clone(),
    })
}

fn parse_strlen(args: &[String]) -> Result<StringCommand> {
    if args.len() != 2 {
        return Err(anyhow!("STRLEN requires exactly 1 argument".to_string()));
    }

    Ok(StringCommand::Strlen {
        key: args[1].clone(),
    })
}

fn parse_incr(args: &[String]) -> Result<StringCommand> {
    if args.len() != 2 {
        return Err(anyhow!("INCR requires exactly 1 argument".to_string()));
    }

    Ok(StringCommand::Incr {
        key: args[1].clone(),
    })
}

fn parse_incrby(args: &[String]) -> Result<StringCommand> {
    if args.len() != 3 {
        return Err(anyhow!("INCRBY requires exactly 2 arguments".to_string(),));
    }

    let increment = args[2]
        .parse::<i64>()
        .map_err(|_| anyhow!("Invalid increment value for INCRBY".to_string()))?;

    Ok(StringCommand::IncrBy {
        key: args[1].clone(),
        increment,
    })
}

fn parse_incrbyfloat(args: &[String]) -> Result<StringCommand> {
    if args.len() != 3 {
        return Err(anyhow!(
            "INCRBYFLOAT requires exactly 2 arguments".to_string(),
        ));
    }

    let increment = args[2]
        .parse::<f64>()
        .map_err(|_| anyhow!("Invalid increment value for INCRBYFLOAT".to_string()))?;

    Ok(StringCommand::IncrByFloat {
        key: args[1].clone(),
        increment,
    })
}

fn parse_decr(args: &[String]) -> Result<StringCommand> {
    if args.len() != 2 {
        return Err(anyhow!("DECR requires exactly 1 argument".to_string()));
    }

    Ok(StringCommand::Decr {
        key: args[1].clone(),
    })
}

fn parse_decrby(args: &[String]) -> Result<StringCommand> {
    if args.len() != 3 {
        return Err(anyhow!("DECRBY requires exactly 2 arguments".to_string(),));
    }

    let decrement = args[2]
        .parse::<i64>()
        .map_err(|_| anyhow!("Invalid decrement value for DECRBY".to_string()))?;

    Ok(StringCommand::DecrBy {
        key: args[1].clone(),
        decrement,
    })
}

fn parse_getrange(args: &[String]) -> Result<StringCommand> {
    if args.len() != 4 {
        return Err(anyhow!("GETRANGE requires exactly 3 arguments".to_string(),));
    }

    let start = args[2]
        .parse::<i64>()
        .map_err(|_| anyhow!("Invalid start value for GETRANGE".to_string()))?;
    let end = args[3]
        .parse::<i64>()
        .map_err(|_| anyhow!("Invalid end value for GETRANGE".to_string()))?;

    Ok(StringCommand::GetRange {
        key: args[1].clone(),
        start,
        end,
    })
}

fn parse_setrange(args: &[String]) -> Result<StringCommand> {
    if args.len() != 4 {
        return Err(anyhow!("SETRANGE requires exactly 3 arguments".to_string(),));
    }

    let offset = args[2]
        .parse::<u64>()
        .map_err(|_| anyhow!("Invalid offset value for SETRANGE".to_string()))?;

    Ok(StringCommand::SetRange {
        key: args[1].clone(),
        offset,
        value: args[3].clone(),
    })
}
