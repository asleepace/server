use super::http_response::HttpResponse;
use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

/**

    Control line feed character denotes the end of a line in HTTP.
    This is used to separate headers and the body of a request.

*/
const CRLF: &str = "\r\n";

pub struct HttpRequest {
    pub response: HttpResponse,
    data: Vec<String>,
}

impl HttpRequest {
    pub fn new() -> Self {
        HttpRequest {
            data: Vec::new(),
            response: HttpResponse::new(),
        }
    }

    pub fn info(&self) {
        println!("[http_request] info:\r\n{:?}", self.data);
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

    pub fn url(&self) -> &str {
        let data = self.data.first().unwrap();
        data.split_whitespace().nth(1).unwrap()
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
