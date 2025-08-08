use super::http_headers::{HttpHeaders, HttpMethod};
use super::http_response::HttpResponse;
use crate::core::error::ServerError;
use crate::core::http::HttpStatus;
use crate::core::server::Flag;
use crate::core::util::get_mime_type;
use crate::core::Path;
use crate::core::ServerEvent;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Result};
use std::io::{BufWriter, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::Arc;

/**

    Control line feed character denotes the end of a line in HTTP.
    This is used to separate headers and the body of a request.

*/
const CRLF: &str = "\r\n";
static mut MAX_REQUEST_LINE: usize = 8 * 1024; // 8 KiB
static mut MAX_HEADERS: usize = 64;
static mut MAX_BODY: usize = 2 * 1024 * 1024; // 2 MiB (cap)

#[derive(Debug, Clone)]
pub enum ResponseState {
    NotHandled,
    Handled(u16),  // status code
    Error(String), // Store error message instead of std::io::Error
}

#[derive(Clone)]
pub struct HttpRequest {
    pub uri: String,
    pub headers: HttpHeaders,
    pub response: HttpResponse,
    pub connection: Option<Arc<TcpStream>>,
    pub data: Vec<String>,
    pub body: Vec<u8>,
    pub params: Option<std::collections::HashMap<String, String>>,
    pub query: Option<std::collections::HashMap<String, String>>,
    pub response_state: ResponseState,
}

impl HttpRequest {
    /**
        Create a new HttpRequest instance with an automatic reference counted TcpStream,
        and read the incoming data from the stream.
    */
    pub fn new(stream: Arc<TcpStream>) -> Self {
        let (data, body) = match HttpRequest::read_stream_data(&stream) {
            Ok(tuple) => tuple,
            Err(error) => {
                println!("[http_request] could not read stream: {:?}", error);
                (Vec::new(), Vec::new())
            }
        };
        let headers = match HttpHeaders::from(&data) {
            None => HttpHeaders::new(),
            Some(headers) => headers,
        };

        let uri = headers.uri.to_string();

        println!("[http_request] new request: {:}", uri);

        let mut req = HttpRequest {
            response: HttpResponse::new(),
            connection: Some(stream),
            headers,
            data,
            body,
            response_state: ResponseState::NotHandled,
            params: None,
            query: None,
            uri,
        };
        if let Some(qidx) = req.uri.find('?') {
            let qs = &req.uri[qidx + 1..];
            let mut map = std::collections::HashMap::new();
            for pair in qs.split('&') {
                let mut it = pair.splitn(2, '=');
                if let (Some(k), Some(v)) = (it.next(), it.next()) {
                    map.insert(k.to_string(), v.to_string());
                }
            }
            req.query = Some(map);
        }
        req
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
            body: Vec::new(),
            response_state: ResponseState::NotHandled,
            params: None,
            query: None,
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
            body: self.body.clone(),
            headers: self.headers.clone(),
            response: self.response.clone(),
            connection: match &self.connection {
                Some(conn) => Some(Arc::clone(conn)),
                None => None,
            },
            response_state: match &self.response_state {
                ResponseState::NotHandled => ResponseState::NotHandled,
                ResponseState::Handled(status) => ResponseState::Handled(*status),
                ResponseState::Error(msg) => ResponseState::Error(msg.clone()),
            },
            params: self.params.clone(),
            query: self.query.clone(),
        }
    }

    /**
        Converts a TcpStream into a byte vector, reads until a CRLF is found.
        or times out after 5 seconds.
    */
    fn read_stream_data(tcp_stream: &TcpStream) -> Result<(Vec<String>, Vec<u8>)> {
        let mut reader = BufReader::new(tcp_stream);
        let mut header = Vec::new();
        loop {
            let mut data = String::new();
            match reader.read_line(&mut data) {
                Ok(bytes) => {
                    if data == CRLF || bytes == 0 {
                        break;
                    } else {
                        // Enforce limits
                        if header.is_empty() && data.len() > unsafe { MAX_REQUEST_LINE } {
                            return Err(Error::new(
                                ErrorKind::InvalidInput,
                                "request line too long",
                            ));
                        }
                        if header.len() >= unsafe { MAX_HEADERS } {
                            return Err(Error::new(ErrorKind::InvalidInput, "too many headers"));
                        }
                        header.push(data);
                    }
                }
                Err(error) => {
                    eprintln!("[http_request] error: {:?}", error);
                    return Err(error);
                }
            }
        }

        // Try to read request body if Content-Length is present
        let mut content_length: usize = 0;
        for line in &header {
            if let Some((name, value)) = HttpHeaders::parse_header(line) {
                if name.eq_ignore_ascii_case("Content-Length") {
                    if let Ok(n) = value.parse::<usize>() {
                        content_length = n;
                        break;
                    }
                }
            }
        }

        let mut body = Vec::new();
        if content_length > 0 {
            body.resize(content_length, 0);
            reader.read_exact(&mut body)?;
        }

        Ok((header, body))
    }

    pub fn set_parser_limits(max_line: usize, max_headers: usize, max_body: usize) {
        unsafe {
            MAX_REQUEST_LINE = max_line;
            MAX_HEADERS = max_headers;
            MAX_BODY = max_body;
        }
    }

    pub fn info(&self) -> String {
        self.headers.info()
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
            // handle this in a block to drop the mutable borrow
            let mut stream = stream_ref.as_ref();
            let bytes = response.prepare();
            stream.write_all(&bytes)?;
            stream.flush()?;
            // do not shutdown here to support keep-alive
        }
        Ok(())
    }

    pub fn serve_static_file(&mut self) -> Result<Flag> {
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
            // do not shutdown here to support keep-alive
        }
        Ok(Flag::StaticFile)
    }

    pub fn url(&self) -> String {
        let raw = self.headers.uri_string();
        match raw.find('?') {
            Some(idx) => raw[..idx].to_string(),
            None => raw,
        }
    }

    pub fn set_params(&mut self, params: std::collections::HashMap<String, String>) {
        self.params = Some(params);
    }

    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.as_ref()?.get(key)
    }

    pub fn query_param(&self, key: &str) -> Option<&String> {
        self.query.as_ref()?.get(key)
    }

    pub fn body_len(&self) -> usize {
        self.body.len()
    }

    pub fn body_bytes(&self) -> &[u8] {
        &self.body
    }

    pub fn body_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    /// Returns true if the current response has been set up as an SSE stream
    pub fn is_event_stream(&self) -> bool {
        match self.response.headers.raw.get("Content-Type") {
            Some(v) => v == "text/event-stream",
            None => false,
        }
    }

    pub fn event_source(&mut self) -> Result<Flag> {
        let result = self.response.start_event_stream();
        let stream_ref = self
            .connection
            .as_ref()
            .ok_or(Error::new(ErrorKind::NotFound, "failed to get tcp stream"))?;
        {
            let mut stream = stream_ref.as_ref();
            let bytes = self.response.prepare();
            stream.write_all(&bytes)?;
            stream.flush()?;
        }
        result
    }

    /**
        Loads a file at the given url and sends it to the client, note that this function
        is generally called by the handler functions.
    */
    pub fn send_file(&mut self, url: &str) -> Result<Flag> {
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
        Ok(Flag::StaticFile)
    }

    /**
        Appends bytes to the body of the response, will return true if the bytes were
        successfully written to the stream. Will return false if the connection is ended,
        or if the stream is not available.
    */
    pub fn append_body_data(&mut self, data: String) -> Result<bool> {
        let bytes = data.into_bytes();
        println!("[http_request] appending body data ({} bytes)", bytes.len());
        match self.connection.as_ref() {
            None => Ok(false),
            Some(stream) => {
                let mut stream = stream.as_ref();
                stream.write_all(&bytes)?;
                stream.flush()?;
                Ok(true)
            }
        }
    }

    pub fn server_side_event(&mut self, event: ServerEvent) -> Result<bool> {
        let mut stream = self
            .connection
            .as_ref()
            .ok_or(ServerError::error("failed to get tcp stream"))?
            .as_ref();

        stream.write_all(&event.to_bytes())?;
        stream.flush()?;
        Ok(true)
    }

    /// Start a chunked transfer-encoding response. Writes status line and headers only.
    pub fn start_chunked(&mut self, content_type: &str) -> Result<Flag> {
        self.response.set_status(HttpStatus::OK);
        self.response.set_header("Transfer-Encoding", "chunked");
        self.response.set_header("Content-Type", content_type);
        let mut bytes = self.response.response_headers().into_bytes();
        let mut stream = self
            .connection
            .as_ref()
            .ok_or(ServerError::error("failed to get tcp stream"))?
            .as_ref();
        stream.write_all(&bytes)?;
        stream.flush()?;
        Ok(Flag::DynamicRoute)
    }

    /// Write a chunk body for an active chunked response.
    pub fn write_chunk(&mut self, data: &[u8]) -> Result<bool> {
        match self.connection.as_ref() {
            None => Ok(false),
            Some(stream) => {
                let mut stream = stream.as_ref();
                let header = format!("{:X}{}", data.len(), CRLF);
                stream.write_all(header.as_bytes())?;
                stream.write_all(data)?;
                stream.write_all(CRLF.as_bytes())?;
                stream.flush()?;
                Ok(true)
            }
        }
    }

    /// Finish a chunked response by sending the terminating chunk.
    pub fn finish_chunked(&mut self) -> Result<bool> {
        match self.connection.as_ref() {
            None => Ok(false),
            Some(stream) => {
                let mut stream = stream.as_ref();
                stream.write_all(b"0\r\n\r\n")?;
                stream.flush()?;
                Ok(true)
            }
        }
    }

    /// Convenience: send a complete text response with content-type and status.
    pub fn send_text(
        &mut self,
        status: HttpStatus,
        content_type: &str,
        body: &str,
    ) -> Result<Flag> {
        let mut response = HttpResponse::new();
        response.set_status(status);
        response.set_body(body.as_bytes().to_vec(), content_type);
        let stream_ref = self
            .connection
            .as_ref()
            .ok_or(ServerError::error("failed to get tcp stream"))?;
        {
            let mut stream = stream_ref.as_ref();
            let bytes = response.prepare();
            stream.write_all(&bytes)?;
            stream.flush()?;
        }
        Ok(Flag::DynamicRoute)
    }

    /// Convenience: send raw bytes with content-type and status.
    pub fn send_bytes(
        &mut self,
        status: HttpStatus,
        content_type: &str,
        body: &[u8],
    ) -> Result<Flag> {
        let mut response = HttpResponse::new();
        response.set_status(status);
        response.set_body(body.to_vec(), content_type);
        let stream_ref = self
            .connection
            .as_ref()
            .ok_or(ServerError::error("failed to get tcp stream"))?;
        {
            let mut stream = stream_ref.as_ref();
            let bytes = response.prepare();
            stream.write_all(&bytes)?;
            stream.flush()?;
        }
        Ok(Flag::DynamicRoute)
    }

    /**
       Close the current connection.
    */
    pub fn close(&self) -> std::io::Result<()> {
        match &self.connection {
            Some(stream) => {
                let mut stream = stream.as_ref();
                stream.shutdown(Shutdown::Both)
            }
            None => Ok(()),
        }
    }

    /// Check if a response has already been sent
    pub fn is_response_sent(&self) -> bool {
        matches!(self.response_state, ResponseState::Handled(_))
    }

    /// Mark that a response has been sent with the given status code
    pub fn mark_response_sent(&mut self, status_code: u16) {
        self.response_state = ResponseState::Handled(status_code);
    }

    /// Mark that an error occurred during request handling
    pub fn mark_error(&mut self, error: std::io::Error) {
        self.response_state = ResponseState::Error(error.to_string());
    }

    /// Get the current response state
    pub fn get_response_state(&self) -> &ResponseState {
        &self.response_state
    }

    /// Check if the request is still pending (no response sent yet)
    pub fn is_pending(&self) -> bool {
        matches!(self.response_state, ResponseState::NotHandled)
    }
}
