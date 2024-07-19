use std::collections::HashMap;
use std::env;

#[derive(Debug)]
pub enum Args {
    Number(u16),
    Text(String),
    Bool(bool),
}

pub fn is_token(token: &str) -> bool {
    token.starts_with("-")
}

pub fn process_args() -> HashMap<String, Args> {
    let args: Vec<String> = env::args().collect();
    let mut i = 0;

    let total_length = args.len() - 1;

    let mut pairs: Vec<(String, Args)> = vec![];

    while i < total_length {
        let token = &args[i];
        let value = &args[i + 1];

        // if no value then it's a boolean flag
        if is_token(value) {
            pairs.push((token.to_owned(), Args::Bool(true)));
            i += 1;
        } else {
            pairs.push((token.to_owned(), Args::Text(value.to_string())));
            i += 2;
        }
    }

    let arguments: HashMap<String, Args> = pairs.into_iter().collect();
    arguments
}

pub fn parse_as_num(args: &HashMap<String, Args>, token: &str) -> Option<u16> {
    match args.get(token) {
        Some(Args::Number(value)) => Some(*value),
        Some(Args::Text(text)) => text.parse::<u16>().ok(),
        Some(Args::Bool(bool)) => match bool {
            true => Some(1),
            false => Some(0),
        },
        _ => None,
    }
}

pub fn parse_as_str(args: &HashMap<String, Args>, token: &str) -> Option<String> {
    match args.get(token) {
        Some(Args::Text(value)) => Some(value.to_owned()),
        Some(Args::Number(num)) => Some(num.to_string()),
        Some(Args::Bool(bool)) => Some(bool.to_string()),
        _ => None,
    }
}

pub fn is_set(args: &HashMap<String, Args>, token: &str) -> bool {
    match args.get(token) {
        Some(Args::Bool(true)) => true,
        _ => false,
    }
}
