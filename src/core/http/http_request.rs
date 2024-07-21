use super::http_response::HttpResponse;
use crate::core::util::get_mime_type;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};
use std::net::{TcpListener, TcpStream};

/**

    Control line feed character denotes the end of a line in HTTP.
    This is used to separate headers and the body of a request.

*/
const CRLF: &str = "\r\n";

#[derive(Clone)]
pub struct HttpRequest<'a> {
    pub response: HttpResponse,
    pub tcp_stream: Option<&'a TcpStream>,
    data: Vec<String>,
}

impl<'a> HttpRequest<'a> {
    pub fn new() -> Self {
        HttpRequest {
            data: Vec::new(),
            tcp_stream: None,
            response: HttpResponse::new(),
        }
    }

    pub fn clone(&self) -> Self {
        println!("cloning request: {:?}", self.data);
        HttpRequest {
            data: self.data.clone(),
            tcp_stream: self.tcp_stream,
            response: self.response.clone(),
        }
    }

    pub fn info(&self) {
        println!("[http_request] info:\r\n{:?}", self.data);
    }

    pub fn set_tcp_stream(&mut self, tcp_stream: &'a TcpStream) {
        self.tcp_stream = Some(tcp_stream);
    }

    pub fn set_data(&mut self, data: Vec<String>) {
        self.data = data;
    }

    pub fn from(tcp_stream: &TcpStream) -> Self {
        let data = match read_stream_data(tcp_stream) {
            Ok(data) => data,
            Err(_) => {
                eprintln!("[http_request] error: failed to read stream data");
                return HttpRequest::new();
            }
        };

        let mut request = HttpRequest::new();
        request.set_data(data);
        request
    }

    pub fn url(&self) -> Option<String> {
        println!("[http_request] url caled: {:?}", self.data);
        if self.data.is_empty() {
            return None;
        }

        match self.data.first() {
            Some(data) => {
                let url = data.split_whitespace().nth(1);
                println!("[http_request] found url: {:?}", url);
                match url {
                    Some(url) => Some(url.to_owned()),
                    None => None,
                }
            }
            None => None,
        }
    }

    /** response api for handlers */

    pub fn get_file(url: &str) -> Result<Vec<u8>, ()> {
        let path = url.trim_matches('/');
        let file_path = format!("./src/public/{}", path);
        return match fs::read(file_path.clone()) {
            Ok(data) => Ok(data),
            Err(_) => {
                eprintln!("[response] file not found: {}", file_path);
                return Result::Err(());
            }
        };
    }

    pub fn send_file(&mut self, url: &str) {
        let mime = get_mime_type(url);

        let response_stream = match self.tcp_stream {
            Some(stream) => stream,
            None => {
                eprintln!("[server] error: failed to send response stream is closed!");
                return;
            }
        };

        match HttpRequest::get_file(url) {
            Ok(data) => {
                let res_data = self
                    .response
                    .append(format!("Content-Length: {:}", data.len()).as_str())
                    .append(format!("Content-Type: {:}", mime).as_str())
                    .append_body(&mut data.to_owned());

                // Step 4: Send the response
                let mut writer = BufWriter::new(response_stream);
                let did_write = writer.write_all(&mut res_data.to_owned());
                match did_write {
                    Ok(_) => println!("[server] response sent!"),
                    Err(error) => eprintln!("[server] error: {:?}", error),
                }

                // Step 5: Close down the writer
                match writer.flush() {
                    Ok(_) => println!("[server] writer flushed!"),
                    Err(error) => eprintln!("[server] writer flush error: {:?}", error),
                };

                // Step 6: Shutdown the stream
                match response_stream.shutdown(std::net::Shutdown::Both) {
                    Ok(_) => println!("[server] stream shutdown!"),
                    Err(error) => eprintln!("[server] stream shutdown error: {:?}", error),
                };
            }
            Err(error) => {
                eprintln!("[server] error: {:?}", error);
            }
        }
    }
}

/**
    Converts a TcpStream into a byte vector, reads until a CRLF is found.
    or times out after 5 seconds.
*/
fn read_stream_data(tcp_stream: &TcpStream) -> Result<Vec<String>, ()> {
    let mut reader = BufReader::new(tcp_stream);
    let mut header = Vec::new();
    loop {
        let mut data = String::new();
        match reader.read_line(&mut data) {
            Ok(bytes) => {
                if data == CRLF || bytes == 0 {
                    break;
                } else {
                    header.push(data);
                }
            }
            Err(error) => {
                eprintln!("[http_request] error: {:?}", error);
                return Err(());
            }
        }
    }

    Ok(header)
}
