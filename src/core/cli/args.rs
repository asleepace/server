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
