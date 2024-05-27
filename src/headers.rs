use std::collections::HashMap;

pub struct Headers {
    headers: HashMap<String, String>,
}

/** @ignore */
pub enum ContentType {
    HTML,
    JSON,
    TEXT,
    From(String),
}

impl Headers {
    pub fn new() -> Self {
        let mut headers = HashMap::new();

        // NOTE: Default headers are set here.
        headers.insert(
            "Content-Type".to_string(),
            "text/plain; charset=utf-8".to_string(),
        );

        Headers { headers }
    }

    pub fn write(&self) -> String {
        let mut headers = "HTTP 1.1 200 OK\r\n".to_string();
        for (header, value) in &self.headers {
            headers.push_str(&format!("{}: {}\r\n", header, value));
        }
        headers
    }

    pub fn set(&mut self, header: String, value: String) {
        self.headers.insert(header, value);
    }

    pub fn get(&self, header: String) -> Option<String> {
        self.headers.get(&header).cloned()
    }

    pub fn set_content_type(&mut self, content_type: ContentType) {
        let header_value = match content_type {
            ContentType::TEXT => "text/plain; charset=utf-8".to_string(),
            ContentType::HTML => "text/html; charset=utf-8".to_string(),
            ContentType::JSON => "application/json".to_string(),
            ContentType::From(value) => value.to_string(),
        };
        self.headers
            .insert("Content-Type".to_string(), header_value.to_string());
    }

    pub fn set_content_length(&mut self, value: usize) {
        self.headers
            .insert("Content-Length".to_string(), value.to_string());
    }
}
