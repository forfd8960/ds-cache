use crate::commands::ListCommand;

use anyhow::{Result, anyhow};

impl ListCommand {
    pub fn from_frame_args(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(anyhow!("Empty command".to_string()));
        }

        let cmd_name = args[0].to_uppercase();
        match cmd_name.as_str() {
            "LPUSH" => parse_lpush(args),
            "RPUSH" => parse_rpush(args),
            "LPOP" => parse_lpop(args),
            "RPOP" => parse_rpop(args),
            "LLEN" => parse_llen(args),
            "LRANGE" => parse_lrange(args),
            _ => Err(anyhow!("Unknown list command: {}", cmd_name)),
        }
    }
}

fn parse_lpush(args: &[String]) -> Result<ListCommand> {
    if args.len() < 3 {
        return Err(anyhow!("LPUSH requires at least 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let values = args[2..].to_vec();

    Ok(ListCommand::LPush { key, values })
}

fn parse_rpush(args: &[String]) -> Result<ListCommand> {
    if args.len() < 3 {
        return Err(anyhow!("RPUSH requires at least 2 arguments".to_string()));
    }

    let key = args[1].clone();
    let values = args[2..].to_vec();

    Ok(ListCommand::RPush { key, values })
}

fn parse_lpop(args: &[String]) -> Result<ListCommand> {
    if args.len() != 2 {
        return Err(anyhow!("LPOP requires exactly 2 argument".to_string()));
    }

    let key = args[1].clone();
    let count = args.get(2).and_then(|s| s.parse::<u64>().ok());
    Ok(ListCommand::LPop { key, count })
}

fn parse_rpop(args: &[String]) -> Result<ListCommand> {
    if args.len() != 2 {
        return Err(anyhow!("RPOP requires exactly 2 argument".to_string()));
    }

    let key = args[1].clone();
    let count = args.get(2).and_then(|s| s.parse::<u64>().ok());
    Ok(ListCommand::RPop { key, count })
}

fn parse_llen(args: &[String]) -> Result<ListCommand> {
    if args.len() != 2 {
        return Err(anyhow!("LLEN requires exactly 1 argument".to_string()));
    }

    let key = args[1].clone();
    Ok(ListCommand::LLen { key })
}

fn parse_lrange(args: &[String]) -> Result<ListCommand> {
    if args.len() != 4 {
        return Err(anyhow!("LRANGE requires exactly 3 arguments".to_string()));
    }

    let key = args[1].clone();
    let start = args[2]
        .parse::<i64>()
        .map_err(|_| anyhow!("Invalid start index".to_string()))?;
    let stop = args[3]
        .parse::<i64>()
        .map_err(|_| anyhow!("Invalid stop index".to_string()))?;

    Ok(ListCommand::LRange { key, start, stop })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lpush() {
        let args = vec![
            "LPUSH".to_string(),
            "mylist".to_string(),
            "value1".to_string(),
            "value2".to_string(),
        ];
        let cmd = parse_lpush(&args).unwrap();
        match cmd {
            ListCommand::LPush { key, values } => {
                assert_eq!(key, "mylist");
                assert_eq!(values, vec!["value1".to_string(), "value2".to_string()]);
            }
            _ => panic!("Expected LPush command"),
        }
    }

    #[test]
    fn test_parse_rpush() {
        let args = vec![
            "RPUSH".to_string(),
            "mylist".to_string(),
            "value1".to_string(),
            "value2".to_string(),
        ];
        let cmd = parse_rpush(&args).unwrap();
        match cmd {
            ListCommand::RPush { key, values } => {
                assert_eq!(key, "mylist");
                assert_eq!(values, vec!["value1".to_string(), "value2".to_string()]);
            }
            _ => panic!("Expected LPush command"),
        }
    }

    #[test]
    fn test_parse_lpop() {
        let args = vec!["LPOP".to_string(), "mylist".to_string(), "2".to_string()];
        let cmd = parse_lpop(&args).unwrap();
        match cmd {
            ListCommand::LPop { key, count } => {
                assert_eq!(key, "mylist");
                assert_eq!(count, Some(2));
            }
            _ => panic!("Expected LPop command"),
        }
    }

    #[test]
    fn test_parse_rpop() {
        let args = vec!["RPOP".to_string(), "mylist".to_string(), "2".to_string()];
        let cmd = parse_rpop(&args).unwrap();
        match cmd {
            ListCommand::RPop { key, count } => {
                assert_eq!(key, "mylist");
                assert_eq!(count, Some(2));
            }
            _ => panic!("Expected RPop command"),
        }
    }
}
