use crate::headers::{ContentType, Headers};
use std::{
    fs,
    io::{Error, Write},
    net::TcpStream,
    result::Result,
};

pub struct Response {
    stream: TcpStream,
    status: u16,
    headers: Headers,
    body: String,
}

impl Response {
    pub fn new(stream: TcpStream) -> Self {
        Response {
            headers: Headers::new(),
            body: String::new(),
            status: 200,
            stream,
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
        self.headers.set_content_length(body.len());
        self.body = body;
    }

    pub fn file(&mut self, file: &str) {
        println!("[response] reading file: {}", file);
        match fs::read_to_string(file) {
            Ok(data) => self.body(data),
            Err(_) => self.status(404),
        }
    }

    /**

        Send the response to the client, consumes the stream.
    */
    pub fn send(&mut self) -> Result<(), Error> {
        let body = self.body.as_str();
        self.headers.set_content_length(body.len());
        let mut response = self.headers.write();
        response.push_str("\r\n");
        response.push_str(body);
        response.push_str("\r\n");

        let bytes = response.as_bytes();
        let mut stream = &self.stream;
        println!("\r\n{:}", response);
        let output = stream.write_all(bytes);
        println!("[response] success: {:}", output.is_ok());
        stream.flush();
        stream.shutdown(std::net::Shutdown::Both);
        output
    }

    fn response_empty(&mut self) -> String {
        let content_length = self.headers.get("Content-Length".to_string());
        if content_length.is_none() {
            self.headers.set_content_length(0);
        }
        let mut empty_response = self.headers.write();
        empty_response.push_str("\r\n");
        empty_response
    }

    // fn response_with_body(&mut self) -> String {
    //     println!(
    //         "[response] response_with_body {}",
    //         self.body.as_ref().unwrap().len()
    //     );
    //     let body = self.body.as_ref().unwrap();
    //     self.headers.set_content_length(body.len());
    //     let mut response = self.headers.write();
    //     response.push_str("\r\n");
    //     response.push_str(body);
    //     response.push_str("\r\n");
    //     response
    // }
}
