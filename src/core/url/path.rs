use std::{collections::HashMap, io::Error, io::ErrorKind, sync::Arc};

type PathParams = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct Path {
    pub path: Arc<str>,
    pub params: Option<PathParams>,
}

impl Path {
    pub fn to_string(&self) -> String {
        match &self.params {
            Some(params) => {
                let params_str = params
                    .iter()
                    .map(|(key, value)| format!("{}={}", key, value))
                    .collect::<Vec<String>>()
                    .join("&");
                format!("{}?{}", self.path, params_str)
            }
            None => self.path.to_string(),
        }
    }

    pub fn from(path_str: &str) -> Self {
        let (path_str, query_str) = split_path_and_params(path_str);
        let path = sanitize_path(path_str);
        let params = parse_query(query_str);
        Path {
            path: path.into(),
            params,
        }
    }
}

/** Split a path and query into a tuple */
pub fn split_path_and_params(path_str: &str) -> (&str, Option<&str>) {
    match path_str.split_once('?') {
        Some((path, query_str)) => (path, Some(query_str)),
        None => (path_str, None),
    }
}

/**
 * Sanitize a path by removing invalid path segments, converting to lowercase,
 * and joining the segments with a forward slash.
 */
pub fn sanitize_path(path: &str) -> String {
    let output = path
        .split("/")
        .filter(|segment| is_valid_path(segment))
        .map(|segment| segment.to_lowercase())
        .collect::<Vec<String>>()
        .join("/");

    if output.is_empty() {
        "/".to_string()
    } else {
        output
    }
}

/**
 * Check if a path segment only contains alphanumeric characters,
 * underscores, or dashes and is not empty.
 */
pub fn is_valid_path(segment: &str) -> bool {
    match segment.is_empty() {
        true => false,
        false => segment
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
    }
}

/**
 * Parse a query string into a HashMap of key-value pairs.
 */
pub fn parse_query(query_str: Option<&str>) -> Option<PathParams> {
    let query = match query_str {
        Some(query_str) => query_str,
        None => return None,
    };

    if query.is_empty() {
        return None;
    }

    let params = query
        .split("&")
        .map(|pair| pair.split_once("="))
        .filter(|pair| pair.is_some())
        .map(|pair| pair.unwrap())
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();

    Some(params)
}
