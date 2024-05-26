use crate::headers::{ContentType, Headers};
use std::{
    io::{Error, Write},
    net::TcpStream,
    option::Option,
    result::Result,
};

pub struct Response {
    status: u16,
    headers: Headers,
    body: Option<String>,
}

impl Response {
    pub fn new(status: u16) -> Self {
        Response {
            headers: Headers::new(),
            body: Option::None,
            status,
        }
    }

    pub fn content_type(&mut self, content_type: ContentType) {
        self.headers.set_content_type(content_type);
    }

    pub fn status(&mut self, status: u16) {
        self.status = status;
    }

    pub fn header(&mut self, header: &str, value: &str) {
        self.headers.set(header.to_string(), value.to_string());
    }

    pub fn body(&mut self, body: String) {
        self.body = Some(body);
    }

    pub fn write(&mut self, stream: &TcpStream) -> Result<(), Error> {
        let response = match self.body {
            Some(ref body) => {
                self.headers.set_content_length(body.len());
                let mut response_with_body = self.headers.write();
                response_with_body.push_str("\r\n");
                response_with_body.push_str(body);
                response_with_body.push_str("\r\n");
                response_with_body
            }
            None => {
                let content_length = self.headers.get("Content-Length".to_string());
                if content_length.is_some() {
                    self.headers.set_content_length(0);
                }
                self.headers.set_content_length(0);
                let mut empty_response = self.headers.write();
                empty_response.push_str("\r\n");
                empty_response
            }
        };
        let bytes = response.as_bytes();
        let mut response_stream = stream.to_owned();
        println!("[response] sent: {:}", response);
        let output = response_stream.write_all(bytes);
        println!("[response] done: {:}", output.is_ok());
        response_stream.flush()
    }
}
