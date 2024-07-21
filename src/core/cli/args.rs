use std::collections::HashMap;
use std::env;
use std::pin::Pin;

#[derive(Debug)]
pub enum Args {
    Program(String),
    Decimal(f64),
    Number(i64),
    Text(String),
    Bool(bool),
}

pub fn is_token(token: &str) -> bool {
    token.starts_with("-")
}

pub fn process_args() -> HashMap<String, Args> {
    let args: Vec<String> = env::args().collect();

    // The arguments to be returned to the caller
    let mut arguments: HashMap<String, Args> = HashMap::new();

    println!("[serveros] args: {:?}", args);

    let total_length = args.len() - 1;
    let mut i = 0;

    // NOTE: The first argument is the command itself
    let program_command = args[0].to_owned();

    // Check program command exists and is not a token, if so we can start at index 1
    // otherwise we start at index 0 to parse the flags.
    if is_token(&program_command) == false {
        arguments.insert("serveros".to_owned(), Args::Text(program_command));
        i = 1;
    };

    while i < total_length {
        let token = &args[i];
        let value = &args[i + 1];
        // if the next value is a token, that means the current token is a flag
        // and should be set to true.
        if is_token(value) {
            arguments.insert(token.to_owned(), Args::Bool(true));
            i += 1;
        } else if value.contains('.') {
            match value.parse::<f64>() {
                Ok(value) => {
                    arguments.insert(token.to_owned(), Args::Decimal(value));
                }
                Err(_) => {
                    arguments.insert(token.to_owned(), Args::Text(value.to_owned()));
                }
            }
        } else if value == "true" || value == "false" {
            let boolean = value.parse::<bool>().unwrap();
            arguments.insert(token.to_owned(), Args::Bool(boolean));
        } else {
            match value.parse::<i64>() {
                Ok(number) => {
                    arguments.insert(token.to_owned(), Args::Number(number));
                }
                Err(_) => {
                    arguments.insert(token.to_owned(), Args::Text(value.to_owned()));
                }
            }
        };

        // increment by 2 to skip the value
        i += 2;
    }

    arguments
}

pub fn parse_as_num(args: &HashMap<String, Args>, token: &str) -> Option<i64> {
    match args.get(token) {
        Some(Args::Number(value)) => Some(*value),
        Some(Args::Decimal(decimal)) => Some(*decimal as i64),
        Some(Args::Text(text)) => text.parse::<i64>().ok(),
        Some(Args::Bool(bool)) => match bool {
            true => Some(1),
            false => Some(0),
        },
        _ => None,
    }
}

pub fn parse_as_float(args: &HashMap<String, Args>, token: &str) -> Option<f64> {
    match args.get(token) {
        Some(Args::Decimal(value)) => Some(*value),
        Some(Args::Number(integer)) => Some(*integer as f64),
        Some(Args::Text(text)) => text.parse::<f64>().ok(),
        Some(Args::Bool(bool)) => match bool {
            true => Some(1.0),
            false => Some(0.0),
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
