use crate::core::file::URI;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum HttpVersion {
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    Name(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    TRACE,
    CONNECT,
    PATCH,
    Name(String),
}

#[derive(Clone, Debug)]
pub struct HttpHeaders {
    pub method: HttpMethod,
    pub version: HttpVersion,
    pub uri: URI,
    pub raw: HashMap<String, String>,
}

impl HttpHeaders {
    pub fn clone(&self) -> HttpHeaders {
        HttpHeaders {
            method: self.method.clone(),
            version: self.version.clone(),
            uri: self.uri.clone(),
            raw: self.raw.clone(),
        }
    }

    pub fn set_version(&mut self, version: HttpVersion) {
        self.version = version;
    }

    pub fn version_from_string(version: &str) -> HttpVersion {
        match version {
            "HTTP/1.0" => HttpVersion::HTTP1_0,
            "HTTP/1.1" => HttpVersion::HTTP1_1,
            "HTTP/2.0" => HttpVersion::HTTP2_0,
            _ => HttpVersion::Name(version.to_string()),
        }
    }

    pub fn method_from_string(method: &str) -> HttpMethod {
        match method.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            "TRACE" => HttpMethod::TRACE,
            "CONNECT" => HttpMethod::CONNECT,
            "PATCH" => HttpMethod::PATCH,
            _ => HttpMethod::Name(method.to_uppercase()),
        }
    }

    pub fn set_content_type(&mut self, content_type: &str) {
        self.raw
            .insert("Content-Type".to_string(), content_type.to_string());
    }

    pub fn set_content_length(&mut self, length: usize) {
        self.raw
            .insert("Content-Length".to_string(), length.to_string());
    }

    pub fn parse_header(header: &str) -> Option<(&str, &str)> {
        let header = header.trim_end_matches("\r\n");
        let mut parts = header.splitn(2, ':');
        let name = parts.next()?.trim();
        let value = parts.next()?.trim();
        Some((name, value))
    }

    pub fn parse_http(line: String) -> Result<(HttpMethod, HttpVersion, URI), ()> {
        let parts = line.split(" ").collect::<Vec<&str>>();
        if parts.len() < 3 {
            println!("[http_headers] failed to parse http headers: {:?}", line);
            return Err(());
        } else {
            let (raw_method, raw_uri, raw_version) = (parts[0], parts[1], parts[2]);
            return Ok((
                HttpHeaders::method_from_string(raw_method),
                HttpHeaders::version_from_string(raw_version),
                URI::new(raw_uri),
            ));
        }
    }

    pub fn new() -> Self {
        HttpHeaders {
            method: HttpMethod::GET,
            version: HttpVersion::HTTP2_0,
            uri: URI::new("/"),
            raw: HashMap::new(),
        }
    }

    pub fn uri_string(&self) -> String {
        self.uri.to_string()
    }

    pub fn method_string(&self) -> String {
        match self.method {
            HttpMethod::GET => "GET".to_string(),
            HttpMethod::POST => "POST".to_string(),
            HttpMethod::PUT => "PUT".to_string(),
            HttpMethod::DELETE => "DELETE".to_string(),
            HttpMethod::HEAD => "HEAD".to_string(),
            HttpMethod::OPTIONS => "OPTIONS".to_string(),
            HttpMethod::TRACE => "TRACE".to_string(),
            HttpMethod::CONNECT => "CONNECT".to_string(),
            HttpMethod::PATCH => "PATCH".to_string(),
            HttpMethod::Name(ref name) => name.to_string(),
        }
    }

    pub fn version_string(&self) -> String {
        match self.version {
            HttpVersion::HTTP1_0 => "HTTP/1.0".to_string(),
            HttpVersion::HTTP1_1 => "HTTP/1.1".to_string(),
            HttpVersion::HTTP2_0 => "HTTP/2.0".to_string(),
            HttpVersion::Name(ref name) => name.to_string(),
        }
    }

    /**
        Parse the raw data into a HttpHeaders struct, this should be the data from the client
        request which has been split by the control line feed.
    */
    pub fn from(data: &Vec<String>) -> Option<Self> {
        // check if data is emtpy
        if data.is_empty() {
            println!("[http_headers] failed to parse http headers: {:?}", data);
            return None;
        }

        // split raw data into the first line and the headers
        let (http_request_info, http_request_headers) = data.split_at(1);

        // parse the first line of the raw data for the http method, version and uri
        // if these are not present then return None as this is an invalid request.
        let http_info = match http_request_info.first() {
            Some(first_line) => match HttpHeaders::parse_http(first_line.to_string()) {
                Ok(info) => info,
                Err(_) => {
                    println!("[http_headers] failed to parse http headers: {:?}", data);
                    return None;
                }
            },
            None => {
                println!("[http_headers] failed to parse http headers: {:?}", data);
                return None;
            }
        };

        // hash map to store the raw headers
        let mut raw = HashMap::new();

        // iterate over the headers and parse them into a key value pair, attempt to
        // insert them into the raw hash map. Ok to skip over invalid entries.
        for line in http_request_headers.iter() {
            match HttpHeaders::parse_header(line) {
                Some((name, value)) => {
                    raw.insert(name.to_string(), value.to_string());
                }
                None => {
                    println!("[http_headers] invalid header {:?}", line);
                }
            }
        }

        Some(HttpHeaders {
            method: http_info.0,
            version: http_info.1,
            uri: http_info.2,
            raw,
        })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.raw.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.raw.insert(key.to_string(), value.to_string());
    }

    pub fn info(&self) -> String {
        let mut info_str = format!("{:?} {:?} {:?}\n", self.method, self.version, self.uri);
        for (key, value) in self.raw.iter() {
            info_str.push_str(&format!("\t{}: {}\n", key, value));
        }
        info_str
    }
}
