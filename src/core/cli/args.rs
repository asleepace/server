use std::env;

#[derive(Debug)]
pub enum ArgType {
    Number(i32),
    Text(String),
    Bool(bool),
}

pub fn is_token(token: &str) -> bool {
    token.starts_with("-")
}

pub fn process_args() -> Vec<(String, ArgType)> {
    let args: Vec<String> = env::args().collect();
    let mut i = 0;

    let total_length = args.len() - 1;

    let mut pairs = vec![];

    while i < total_length {
        let token = &args[i];
        let value = &args[i + 1];

        // if no value then it's a boolean flag
        if is_token(value) {
            pairs.push((token.to_owned(), ArgType::Bool(true)));
            i += 1;
        } else {
            pairs.push((token.to_owned(), ArgType::Text(value.to_string())));
            i += 2;
        }
    }
    println!("{:?}", pairs);
    pairs
}
