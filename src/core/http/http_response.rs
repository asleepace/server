use crate::core::http::http_headers::HttpHeaders;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub headers: HttpHeaders,
    pub version: String,
    pub status: String,
    pub code: u16,
    data: String,
    pub body: Option<Vec<u8>>,
}

static CRLF: &str = "\r\n";

fn with_crlf(data: &str) -> String {
    format!("{}{}", data, CRLF)
}

impl HttpResponse {
    /**
        Control line feed character denotes the end of a line in HTTP.
        This is used to separate headers and the body of a request.
    */
    pub fn with_crlf(data: &str) -> String {
        format!("{}{}", data, CRLF)
    }

    pub fn clone(&self) -> HttpResponse {
        HttpResponse {
            headers: self.headers.clone(),
            version: self.version.clone(),
            status: self.status.clone(),
            code: self.code,
            data: self.data.clone(),
            body: self.body.clone(),
        }
    }

    pub fn new() -> Self {
        HttpResponse {
            headers: HttpHeaders::new(),
            version: String::from("HTTP/1.1"),
            status: String::from("OK"),
            data: String::new(),
            code: 200,
            body: None,
        }
    }

    pub fn set_body(&mut self, body: Vec<u8>, mime: &str) {
        self.headers.set_content_length(body.len());
        self.headers.set_content_type(mime);
        self.body = Some(body);
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.set(key, value);
    }

    pub fn set_code(&mut self, code: u16) -> &mut Self {
        self.code = code;
        self
    }

    pub fn append(&mut self, data: &str) -> &mut Self {
        self.data.push_str(with_crlf(data).as_str());
        self
    }

    /**
        Prepare the response to be sent to the client. The request has 3 parts:
            1. The status line
            2. The headers
            3. The body
    */
    pub fn prepare(&mut self) -> Vec<u8> {
        let mut response = String::new();
        let header_start = format!("{} {} {}", self.version, self.code, self.status);
        response.push_str(header_start.as_str());

        for (key, value) in self.headers.raw.iter() {
            let header_line = format!("{}: {}{}", key, value, CRLF);
            response.push_str(header_line.as_str());
        }

        response.push_str(CRLF);
        response.push_str(CRLF);

        let mut response_bytes = response.into_bytes();

        if let Some(mut body) = self.body.take() {
            response_bytes.append(&mut body);
        }

        response_bytes
    }

    /**
        Send the response to the client. This function takes a mutable reference to a TcpStream
        which is used to send the response to the client. NOTE: This will close the connection
        after sending the response.
    */
    pub fn send(&mut self, tcp_stream: &mut TcpStream) -> std::io::Result<()> {
        let response_in_bytes = self.prepare();
        tcp_stream.write_all(&response_in_bytes)?;
        tcp_stream.flush()?;
        tcp_stream.shutdown(Shutdown::Both)?;
        Ok(())
    }

    fn http_headers(&self) -> String {
        let headers = format!("{} {} {}", self.version, self.code, self.status);
        with_crlf(headers.as_str())
    }

    pub fn append_body(&self, body: &mut Vec<u8>) -> Vec<u8> {
        let mut response = String::new();
        let http_headers = self.http_headers();
        response.push_str(http_headers.as_str());
        response.push_str(CRLF);
        response.push_str(CRLF);
        let mut response_bytes = response.into_bytes();
        response_bytes.append(body);
        response_bytes
    }

    pub fn send_body(&mut self, mime: &str, data: &Vec<u8>, tcp_stream: &mut TcpStream) {
        let res_data = self
            .append(format!("Content-Length: {:}", data.len()).as_str())
            .append(format!("Content-Type: {:}", mime).as_str())
            .append_body(&mut data.to_owned());

        // Step 4: Send the response
        let mut writer = BufWriter::new(tcp_stream);
        let did_write = writer.write_all(&mut res_data.to_owned());
        match did_write {
            Ok(_) => println!("[server] response sent!"),
            Err(error) => eprintln!("[server] error: {:?}", error),
        }
    }
}

/**
    Create a new HTTP response which can be used to send data back to the client.
    Below is an example of how to use this function:
    ```
    use http::http_response::create_http_response;
    let response = create_http_response();
    response.append("Content-Type: text/html");
    ```
*/
pub fn create_http_response() -> HttpResponse {
    let mut response = HttpResponse::new();
    response
        .append("Content-Type: */*")
        .append("<h1>Hello, World!</h1>");

    response
}
