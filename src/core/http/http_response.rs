use crate::core::http::http_headers::HttpHeaders;
use crate::core::util::get_mime_type;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::fs;
use std::io::Error;
use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

use super::HttpStatus;

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub headers: HttpHeaders,
    pub version: String,
    pub status: HttpStatus,
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
            body: self.body.clone(),
        }
    }

    pub fn new() -> Self {
        HttpResponse {
            headers: HttpHeaders::new(),
            version: String::from("HTTP/1.1"),
            status: HttpStatus::OK,
            body: None,
        }
    }

    /**
        Create a new HttpResponse instance with a static file which is ready to be sent.

    */
    pub fn with_static_file(url: &str) -> Result<Self, Error> {
        let mut response = HttpResponse {
            headers: HttpHeaders::new(),
            version: String::from("HTTP/1.1"),
            status: HttpStatus::OK,
            body: None,
        };
        let (file_bytes, file_type) = HttpResponse::get_file(url)?;
        println!("[http_response] setting body ({:})", file_type);
        response.set_body(file_bytes, file_type.as_str());
        Ok(response)
    }

    pub fn get_file(url: &str) -> Result<(Vec<u8>, String), Error> {
        let path = url.trim_matches('/');
        let file_path = format!("./src/public/{}", path);
        println!("[http_request] get file {:?}", file_path);
        let data = fs::read(file_path)?;
        let mime = get_mime_type(url);
        println!(
            "[http_request] success getting {:?} ({:}) ({:})",
            path,
            mime,
            data.len()
        );
        Ok((data, mime))
    }

    pub fn set_body(&mut self, body: Vec<u8>, mime: &str) {
        self.headers.set_content_length(body.len());
        self.headers.set_content_type(mime);
        self.body = Some(body);
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.set(key, value);
    }

    pub fn set_status(&mut self, status: HttpStatus) {
        self.status = status;
    }

    // pub fn append(&mut self, data: &str) -> &mut Self {
    //     self.data.push_str(with_crlf(data).as_str());
    //     self
    // }

    /**
        Prepare the response to be sent to the client. The request has 3 parts:
            1. The status line
            2. The headers
            3. The body
    */
    pub fn prepare(&mut self) -> Vec<u8> {
        let mut response = String::new();
        response.push_str(&self.http_headers());
        for (key, value) in self.headers.raw.iter() {
            let header_line = format!("{}: {}{}", key, value, CRLF);
            response.push_str(header_line.as_str());
        }
        response.push_str(CRLF);
        let mut bytes = response.into_bytes();
        if let Some(mut body) = self.body.take() {
            bytes.append(&mut body);
        }
        bytes
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

    /**
        Prepare the status line of the response. The status line is the first line of the response
        and contains the HTTP version, the status code and the status message. Contains CRLF.
    */
    fn http_headers(&self) -> String {
        format!(
            "{} {} {}{}",
            self.version,
            self.status.code(),
            self.status.message(),
            CRLF
        )
    }

    // pub fn append_body(&self, body: &mut Vec<u8>) -> Vec<u8> {
    //     let mut response = String::new();
    //     let http_headers = self.http_headers();
    //     response.push_str(http_headers.as_str());
    //     response.push_str(CRLF);
    //     response.push_str(CRLF);
    //     let mut response_bytes = response.into_bytes();
    //     response_bytes.append(body);
    //     response_bytes
    // }

    // pub fn send_body(&mut self, mime: &str, data: &Vec<u8>, tcp_stream: &mut TcpStream) {
    //     let res_data = self
    //         .append(format!("Content-Length: {:}", data.len()).as_str())
    //         .append(format!("Content-Type: {:}", mime).as_str())
    //         .append_body(&mut data.to_owned());

    //     // Step 4: Send the response
    //     let mut writer = BufWriter::new(tcp_stream);
    //     let did_write = writer.write_all(&mut res_data.to_owned());
    //     match did_write {
    //         Ok(_) => println!("[server] response sent!"),
    //         Err(error) => eprintln!("[server] error: {:?}", error),
    //     }
    // }
}
