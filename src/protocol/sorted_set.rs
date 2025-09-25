use crate::commands::{SortedSetCommand, ZAddCondition, ZAddOptions, ZRangeOptions};
use anyhow::{Result, anyhow};

impl SortedSetCommand {
    pub fn from_frame_args(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(anyhow!("Empty command".to_string()));
        }

        let cmd_name = args[0].to_uppercase();
        match cmd_name.as_str() {
            "ZADD" => parse_zadd(args),
            "ZREM" => parse_zrem(args),
            "ZRANGE" => parse_zrange(args),
            "ZCARD" => parse_zcard(args),
            "ZSCORE" => parse_zscore(args),
            _ => Err(anyhow!("Unknown list command: {}", cmd_name)),
        }
    }
}

fn parse_zadd(args: &[String]) -> Result<SortedSetCommand> {
    if args.len() < 4 || args.len() % 2 != 0 {
        return Err(anyhow!(
            "ZADD requires at least 3 arguments and even number of arguments".to_string()
        ));
    }

    let key = args[1].clone();
    let mut options = ZAddOptions::default();
    let mut start_index = 2;

    // Check for options
    if args[2].to_uppercase() == "NX" {
        options.condition = Some(ZAddCondition::Nx);
        start_index += 1;
    } else if args[2].to_uppercase() == "XX" {
        options.condition = Some(ZAddCondition::Xx);
        start_index += 1;
    }

    let mut pairs = Vec::new();
    for i in (start_index..args.len()).step_by(2) {
        let score = args[i]
            .parse::<f64>()
            .map_err(|_| anyhow!("Invalid score value: {}", args[i]))?;
        let member = args[i + 1].clone();
        pairs.push((score, member));
    }

    Ok(SortedSetCommand::ZAdd {
        key,
        options,
        members: pairs,
    })
}

fn parse_zrem(args: &[String]) -> Result<SortedSetCommand> {
    if args.len() < 3 {
        return Err(anyhow!("ZREM requires at least 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let members = args[2..].to_vec();

    Ok(SortedSetCommand::ZRem { key, members })
}

fn parse_zcard(args: &[String]) -> Result<SortedSetCommand> {
    if args.len() != 2 {
        return Err(anyhow!("ZCARD requires exactly 1 argument".to_string()));
    }

    let key = args[1].clone();
    Ok(SortedSetCommand::ZCard { key })
}

fn parse_zscore(args: &[String]) -> Result<SortedSetCommand> {
    if args.len() != 3 {
        return Err(anyhow!("ZSCORE requires exactly 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let member = args[2].clone();

    Ok(SortedSetCommand::ZScore { key, member })
}

fn parse_zrange(args: &[String]) -> Result<SortedSetCommand> {
    if args.len() < 4 {
        return Err(anyhow!("ZRANGE requires at least 3 arguments".to_string()));
    }

    let key = args[1].clone();
    let start = args[2]
        .parse::<i64>()
        .map_err(|_| anyhow!("Invalid start index".to_string()))?;
    let stop = args[3]
        .parse::<i64>()
        .map_err(|_| anyhow!("Invalid stop index".to_string()))?;

    let mut options = ZRangeOptions::default();
    for arg in &args[4..] {
        match arg.to_uppercase().as_str() {
            "WITHSCORES" => options.with_scores = true,
            _ => return Err(anyhow!("Unknown ZRANGE option: {}", arg)),
        }
    }

    Ok(SortedSetCommand::ZRange {
        key,
        start,
        stop,
        options,
    })
}
