use super::http_headers::{HttpHeaders, HttpMethod};
use super::http_response::HttpResponse;
use crate::core::error::ServerError;
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
    pub headers: HttpHeaders,
    pub response: HttpResponse,
    pub connection: Option<Arc<TcpStream>>,
    data: Vec<String>,
}

impl HttpRequest {
    /**
        Create a new HttpRequest instance with an automatic reference counted TcpStream.
    */
    pub fn new(stream: Arc<TcpStream>) -> Self {
        HttpRequest {
            headers: HttpHeaders::new(),
            response: HttpResponse::new(),
            connection: Some(stream),
            data: Vec::new(),
        }
    }

    pub fn clone(&self) -> Self {
        println!("[http_request] cloning request: {:?}", self.data);
        HttpRequest {
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
        Create a new HttpRequest instance from a TcpStream. NOTE: This is the primary
        entry point for creating a new HttpRequest instance.
    */
    pub fn with(tcp_stream: Arc<TcpStream>) -> Result<Self> {
        let data_stream = match tcp_stream.try_clone() {
            Ok(stream) => stream,
            Err(error) => {
                println!("[http_request] invalid request: {:?}", error);
                return Err(error);
            }
        };

        let data = match HttpRequest::read_stream_data(&data_stream) {
            Ok(data) => data,
            Err(_) => return Err(ServerError::error("failed to read data")),
        };

        let headers = match HttpHeaders::from(&data) {
            Some(headers) => headers,
            None => HttpHeaders::new(),
        };

        let mut request = HttpRequest::new(tcp_stream);
        request.set_headers(headers);
        request.set_data(data);
        Ok(request)
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
        println!("[http_request] info:");
        println!("\t{:?}", self.headers);
    }

    // pub fn set_tcp_stream(&mut self, tcp_stream: TcpStream) {
    //     self.tcp_stream = Some(tcp_stream);
    // }

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

    pub fn serve_static_file(&mut self) -> Result<()> {
        let file_url = self
            .url()
            .ok_or(Error::new(ErrorKind::NotFound, "failed to get url"))?;
        let mut response = HttpResponse::with_static_file(file_url)?;
        let bytes = response.prepare();
        let mut stream = self
            .connection
            .as_ref()
            .ok_or(Error::new(ErrorKind::NotFound, "failed to get tcp stream"))?
            .as_ref();
        stream.write_all(&bytes)?;
        stream.flush()?;
        stream.shutdown(Shutdown::Both)?;
        Ok(())
    }

    pub fn url(&self) -> Option<String> {
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

    // pub fn get_file(url: &str) -> Result<(Vec<u8>, String), Error> {
    //     let path = url.trim_matches('/');
    //     let file_path = format!("./src/public/{}", path);
    //     println!("[http_request] file_path {:?}", file_path);
    //     let data = fs::read(file_path)?;
    //     let mime = get_mime_type(url);
    //     Ok((data, mime))
    // }

    /**
        Loads a file at the given url and sends it to the client, note that this function
        is generally called by the handler functions.
    */
    pub fn send_file(&mut self, url: &str) -> Result<()> {
        let mut response = HttpResponse::with_static_file(url.to_string())?;
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

    // pub fn send_file(&mut self, url: &str) -> Result<(), Error> {
    //     let mut tcp_stream = match &self.connection {
    //         Some(stream) => stream.as_ref(),
    //         None => {
    //             eprintln!("[http_request] error: failed to get tcp stream");
    //             return Err(ServerError::error("failed to get tcp stream"));
    //         }
    //     };

    //     let mime = get_mime_type(url);

    //     return match HttpRequest::get_file(url) {
    //         Ok(data) => {
    //             let response_bytes = self
    //                 .response
    //                 .append(format!("Content-Length: {:}", data.len()).as_str())
    //                 .append(format!("Content-Type: {:}", mime).as_str())
    //                 .append_body(&mut data.to_owned());
    //             tcp_stream.write_all(&response_bytes)?;
    //             tcp_stream.flush()?;
    //             tcp_stream.shutdown(Shutdown::Both)?;
    //             Ok(())
    //         }
    //         Err(error) => {
    //             eprintln!("[server] error: {:?}", error);
    //             return Err(error);
    //         }
    //     };
    // }
}
