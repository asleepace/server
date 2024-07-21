use super::http_headers::{HttpHeaders, HttpMethod};
use super::http_response::HttpResponse;
use crate::core::error::ServerError;
use crate::core::http::HttpStatus;
use crate::core::util::get_mime_type;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result};
use std::io::{BufWriter, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::Arc;

/**

    Control line feed character denotes the end of a line in HTTP.
    This is used to separate headers and the body of a request.

*/
const CRLF: &str = "\r\n";

#[derive(Clone)]
pub struct HttpRequest {
    pub uri: String,
    pub headers: HttpHeaders,
    pub response: HttpResponse,
    pub connection: Option<Arc<TcpStream>>,
    data: Vec<String>,
}

impl HttpRequest {
    /**
        Create a new HttpRequest instance with an automatic reference counted TcpStream,
        and read the incoming data from the stream.
    */
    pub fn new(stream: Arc<TcpStream>) -> Self {
        let data = match HttpRequest::read_stream_data(&stream) {
            Ok(data) => data,
            Err(error) => {
                println!("[http_request] could not read stream: {:?}", error);
                Vec::new()
            }
        };
        let headers = match HttpHeaders::from(&data) {
            None => HttpHeaders::new(),
            Some(headers) => headers,
        };
        HttpRequest {
            uri: headers.uri.to_string(),
            response: HttpResponse::new(),
            connection: Some(stream),
            headers,
            data,
        }
    }

    /**
       Create a new HttpRequest instance with empty headers, response, and data.
    */
    pub fn to(uri: &str) -> Self {
        HttpRequest {
            uri: uri.to_string(),
            headers: HttpHeaders::new(),
            response: HttpResponse::new(),
            connection: None,
            data: Vec::new(),
        }
    }

    /**
        Create a new HttpRequest instance from a TcpStream reference, this will read the incoming
        headers and data from the stream.
    */
    pub fn from(tcp_stream: Arc<TcpStream>) -> Result<Self> {
        Ok(HttpRequest::new(tcp_stream))
    }

    /**
        Clone the current HttpRequest instance and TcpStream.
    */
    pub fn clone(&self) -> Self {
        println!("[http_request] cloning request: {:?}", self.data);
        HttpRequest {
            uri: self.uri.clone(),
            data: self.data.clone(),
            headers: self.headers.clone(),
            response: self.response.clone(),
            connection: match &self.connection {
                Some(conn) => Some(Arc::clone(conn)),
                None => None,
            },
        }
    }

    /**
        Converts a TcpStream into a byte vector, reads until a CRLF is found.
        or times out after 5 seconds.
    */
    fn read_stream_data(tcp_stream: &TcpStream) -> Result<Vec<String>> {
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
                    return Err(error);
                }
            }
        }

        Ok(header)
    }

    pub fn info(&self) {
        println!(
            "[http_request] {:}: {:}",
            self.headers.method_string(),
            self.headers.uri_string(),
        );
        println!("\t{:}", self.headers.info());
    }

    pub fn set_tcp_stream(&mut self, tcp_stream: Arc<TcpStream>) {
        self.connection = Some(tcp_stream);
    }

    pub fn set_data(&mut self, data: Vec<String>) {
        self.data = data;
    }

    pub fn set_headers(&mut self, headers: HttpHeaders) {
        self.headers = headers;
    }

    pub fn is_file_request(&self) -> bool {
        if self.headers.method != HttpMethod::GET {
            return false;
        }
        if self.headers.uri.is_file() == false {
            return false;
        }
        return true;
    }

    pub fn send_404(&mut self) -> Result<()> {
        let mut response = HttpResponse::new();
        let (body, mime) = HttpResponse::get_file("404.html")?;
        response.set_status(HttpStatus::NotFound);
        response.set_body(body, &mime);
        let stream_ref = self
            .connection
            .as_ref()
            .ok_or(ServerError::error("failed to get tcp stream"))?;
        {
            // hnadle this in a block to drop the mutable borrow
            let mut stream = stream_ref.as_ref();
            let bytes = response.prepare();
            stream.write_all(&bytes)?;
            stream.flush()?;
            stream.shutdown(Shutdown::Both)?;
        }
        Ok(())
    }

    pub fn serve_static_file(&mut self) -> Result<()> {
        let file_url = self.url();
        let mut response = HttpResponse::with_static_file(&file_url)?;
        let bytes = response.prepare();
        let stream = self
            .connection
            .as_ref()
            .ok_or(Error::new(ErrorKind::NotFound, "failed to get tcp stream"))?;
        {
            let mut stream = stream.as_ref();
            stream.write_all(&bytes)?;
            stream.flush()?;
            stream.shutdown(Shutdown::Both)?;
        }
        Ok(())
    }

    pub fn url(&self) -> String {
        self.headers.uri.to_string()
    }

    /**
        Loads a file at the given url and sends it to the client, note that this function
        is generally called by the handler functions.
    */
    pub fn send_file(&mut self, url: &str) -> Result<()> {
        let mut response = HttpResponse::with_static_file(url)?;
        let stream_ref = self
            .connection
            .as_ref()
            .ok_or(ServerError::error("failed to get tcp stream"))?;
        {
            // hnadle this in a block to drop the mutable borrow
            let mut stream = stream_ref.as_ref();
            let bytes = response.prepare();
            stream.write_all(&bytes)?;
            stream.flush()?;
            stream.shutdown(Shutdown::Both)?;
        }
        Ok(())
    }
}
