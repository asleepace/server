use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    net::TcpStream,
};

/**
    Represents the end of the headers in an HTTP request.
*/
static REQUEST_END_OF_HEADERS: &str = "\r\n";
static REQUEST_HTTP_VERSION: &str = "http_version";
static REQUEST_METHOD: &str = "method";
static REQUEST_URL: &str = "url";

/**
    Represents the headers of an HTTP request.
    Contains a hashmap of key-value pairs.
*/
pub struct Request {
    data: HashMap<String, String>,
}

impl Request {
    pub fn new(data: HashMap<String, String>) -> Self {
        Request { data }
    }

    pub fn init(tcp_stream: &TcpStream) -> Self {
        let mut reader = BufReader::new(tcp_stream);
        let mut data = HashMap::new();
        let mut is_first_line = true;
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if line == REQUEST_END_OF_HEADERS {
                        break;
                    } else if is_first_line {
                        read_line_http(&mut data, &line);
                        is_first_line = false;
                    } else {
                        read_line_into(&mut data, &line);
                    }
                }
                Err(e) => {
                    eprintln!("[stream] error: {:?}", e);
                    break;
                }
            }
        }

        return Request::new(data);
    }

    pub fn url(&self) -> Option<String> {
        let value = self.data.get(REQUEST_URL);
        match value {
            Some(value) => {
                if value == "/" {
                    return Some("/index.html".to_string());
                }

                Some(value.clone())
            }
            None => None,
        }
    }

    pub fn method(&self) -> Option<&String> {
        self.data.get(REQUEST_METHOD)
    }

    pub fn http_version(&self) -> Option<&String> {
        self.data.get(REQUEST_HTTP_VERSION)
    }

    pub fn info(&self) {
        println!("[Request] http_version: {:?}", self.http_version());
        println!("[Request] method: {:?}", self.method());
        println!("[Request] url: {:?}", self.url());
        for (key, value) in &self.data {
            eprintln!("[Request] {:}: {:}", key, value);
        }
    }
}

/**
    Reads a line from an HTTP request and inserts it into the headers hashmap.
*/
fn read_line_into(headers: &mut HashMap<String, String>, line: &String) {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        eprintln!("[RequestHeaders] failed parsing: {:}", line);
        return;
    }
    let key = parts[0].trim().to_string();
    let value = parts[1].trim().to_string();
    headers.insert(key, value);
}

/**
    Reads the first line of an HTTP request.
    The first line contains the method, URL, and HTTP version.
*/
fn read_line_http(headers: &mut HashMap<String, String>, line: &String) {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() != 3 {
        eprintln!("[RequestHeaders] failed parsing: {:}", line);
        return;
    }
    let method = parts[0].to_string();
    let url = parts[1].to_string();
    let http_version = parts[2].to_string();
    headers.insert(REQUEST_HTTP_VERSION.to_string(), http_version);
    headers.insert(REQUEST_METHOD.to_string(), method);
    headers.insert(REQUEST_URL.to_string(), url);
}
