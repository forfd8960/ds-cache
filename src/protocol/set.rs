use crate::commands::SetCommand;

use anyhow::{Result, anyhow};

impl SetCommand {
    pub fn from_frame_args(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(anyhow!("Empty command".to_string()));
        }

        let cmd_name = args[0].to_uppercase();
        match cmd_name.as_str() {
            "SADD" => parse_sadd(args),
            "SREM" => parse_srem(args),
            "SMEMBERS" => parse_smembers(args),
            "SCARD" => parse_scard(args),
            "SISMEMBER" => parse_sismember(args),
            _ => Err(anyhow!("Unknown set command: {}", cmd_name)),
        }
    }
}

fn parse_sadd(args: &[String]) -> Result<SetCommand> {
    if args.len() < 3 {
        return Err(anyhow!("SADD requires at least 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let members = args[2..].to_vec();

    Ok(SetCommand::SAdd { key, members })
}

fn parse_srem(args: &[String]) -> Result<SetCommand> {
    if args.len() < 3 {
        return Err(anyhow!("SREM requires at least 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let members = args[2..].to_vec();

    Ok(SetCommand::SRem { key, members })
}

fn parse_smembers(args: &[String]) -> Result<SetCommand> {
    if args.len() != 2 {
        return Err(anyhow!("SMEMBERS requires exactly 1 argument".to_string()));
    }

    let key = args[1].clone();
    Ok(SetCommand::SMembers { key })
}

fn parse_scard(args: &[String]) -> Result<SetCommand> {
    if args.len() != 2 {
        return Err(anyhow!("SCARD requires exactly 1 argument".to_string()));
    }

    let key = args[1].clone();
    Ok(SetCommand::SCard { key })
}

fn parse_sismember(args: &[String]) -> Result<SetCommand> {
    if args.len() != 3 {
        return Err(anyhow!(
            "SISMEMBER requires exactly 2 arguments".to_string()
        ));
    }

    let key = args[1].clone();
    let member = args[2].clone();
    Ok(SetCommand::SIsMember { key, member })
}
