use crate::commands::HashCommand;

use anyhow::{Result, anyhow};

impl HashCommand {
    pub fn from_frame_args(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(anyhow!("Empty command".to_string()));
        }

        let cmd_name = args[0].to_uppercase();
        match cmd_name.as_str() {
            "HSET" => parse_hset(args),
            "HGET" => parse_hget(args),
            "HDEL" => parse_hdel(args),
            "HGETALL" => parse_hgetall(args),
            "HLEN" => parse_hlen(args),
            "HMSET" => parse_hmset(args),
            "HMGET" => parse_hmget(args),
            "HEXISTS" => parse_hexists(args),
            "HKEYS" => parse_hkeys(args),
            "HVALS" => parse_hvals(args),
            _ => Err(anyhow!("Unknown list command: {}", cmd_name)),
        }
    }
}

fn parse_hset(args: &[String]) -> Result<HashCommand> {
    if args.len() < 4 || args.len() % 2 != 0 {
        return Err(anyhow!(
            "HSET requires at least 3 arguments and even number of arguments".to_string()
        ));
    }

    let key = args[1].clone();
    let mut pairs = Vec::new();
    for i in (2..args.len()).step_by(2) {
        let field = args[i].clone();
        let value = args[i + 1].clone();
        pairs.push((field, value));
    }

    Ok(HashCommand::HSet { key, pairs })
}
fn parse_hget(args: &[String]) -> Result<HashCommand> {
    if args.len() != 3 {
        return Err(anyhow!("HGET requires exactly 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let field = args[2].clone();

    Ok(HashCommand::HGet { key, field })
}
fn parse_hdel(args: &[String]) -> Result<HashCommand> {
    if args.len() < 3 {
        return Err(anyhow!("HDEL requires at least 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let fields = args[2..].to_vec();

    Ok(HashCommand::HDel { key, fields })
}

fn parse_hgetall(args: &[String]) -> Result<HashCommand> {
    if args.len() != 2 {
        return Err(anyhow!("HGETALL requires exactly 1 argument".to_string()));
    }

    let key = args[1].clone();
    Ok(HashCommand::HGetAll { key })
}

fn parse_hlen(args: &[String]) -> Result<HashCommand> {
    if args.len() != 2 {
        return Err(anyhow!("HLEN requires exactly 1 argument".to_string()));
    }

    let key = args[1].clone();
    Ok(HashCommand::HLen { key })
}

fn parse_hmset(args: &[String]) -> Result<HashCommand> {
    if args.len() < 4 || args.len() % 2 != 0 {
        return Err(anyhow!(
            "HMSET requires at least 3 arguments and even number of arguments".to_string()
        ));
    }

    let key = args[1].clone();
    let mut pairs = Vec::new();
    for i in (2..args.len()).step_by(2) {
        let field = args[i].clone();
        let value = args[i + 1].clone();
        pairs.push((field, value));
    }

    Ok(HashCommand::HMSet { key, pairs })
}

fn parse_hmget(args: &[String]) -> Result<HashCommand> {
    if args.len() < 3 {
        return Err(anyhow!("HMGET requires at least 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let fields = args[2..].to_vec();

    Ok(HashCommand::HMGet { key, fields })
}
fn parse_hexists(args: &[String]) -> Result<HashCommand> {
    if args.len() != 3 {
        return Err(anyhow!("HEXISTS requires exactly 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let field = args[2].clone();

    Ok(HashCommand::HExists { key, field })
}

fn parse_hkeys(args: &[String]) -> Result<HashCommand> {
    if args.len() != 2 {
        return Err(anyhow!("HKEYS requires exactly 1 argument".to_string()));
    }
    let key = args[1].clone();
    Ok(HashCommand::HKeys { key })
}

fn parse_hvals(args: &[String]) -> Result<HashCommand> {
    if args.len() != 2 {
        return Err(anyhow!("HVALS requires exactly 1 argument".to_string()));
    }
    let key = args[1].clone();
    Ok(HashCommand::HVals { key })
}
