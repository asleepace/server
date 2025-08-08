use crate::core::http::http_headers::HttpHeaders;
use crate::core::security::{sanitize_path, validate_path_bounds};
use crate::core::util::get_mime_type;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs;
use std::io::Error;
use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use super::http_headers::HttpVersion;
use super::HttpStatus;
use crate::core::server::Flag;

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub headers: HttpHeaders,
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
            status: self.status.clone(),
            body: self.body.clone(),
        }
    }

    pub fn new() -> Self {
        HttpResponse {
            headers: HttpHeaders::new(),
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
            status: HttpStatus::OK,
            body: None,
        };
        let (file_bytes, file_type) = HttpResponse::get_file(url)?;
        response.set_body(file_bytes, file_type.as_str());
        Ok(response)
    }

    /// Securely resolves a file path with fallback logic:
    /// 1. Try {path}.html if path doesn't end with .html
    /// 2. Try {path}/index.html if primary fails
    /// 3. Return 404.html if both fail
    /// Includes comprehensive security validation to prevent path traversal attacks.
    pub fn resolve_file_path(url: &str) -> Result<(PathBuf, String), Error> {
        // Security: Sanitize the input path
        let clean_path = match sanitize_path(url) {
            Ok(path) => path,
            Err(_security_err) => {
                eprintln!("[security] Blocked request for: {}", url);
                return Self::get_404_fallback();
            }
        };

        let base_dir = Path::new("./src/public");
        
        // Handle root path
        if clean_path.is_empty() {
            let index_path = base_dir.join("index.html");
            if let Err(_security_err) = validate_path_bounds(&index_path, base_dir) {
                eprintln!("[security] Path bounds violation for root path");
                return Self::get_404_fallback();
            }
            return Ok((index_path, get_mime_type("index.html")));
        }

        // Try different resolution strategies in order
        let candidates = Self::generate_path_candidates(&clean_path);
        
        for candidate in candidates {
            let full_path = base_dir.join(&candidate);
            
            // Security: Validate path bounds
            if let Err(_) = validate_path_bounds(&full_path, base_dir) {
                continue; // Skip this candidate, try next
            }
            
            // Check if file exists
            if full_path.exists() && full_path.is_file() {
                println!("[http_request] resolved: {:?}", full_path);
                let mime = get_mime_type(&candidate);
                return Ok((full_path, mime));
            }
        }

        // All candidates failed, return 404.html
        Self::get_404_fallback()
    }

    /// Generates candidate paths to try for file resolution
    fn generate_path_candidates(clean_path: &str) -> Vec<String> {
        let mut candidates = Vec::new();
        
        // Strategy 1: Try exact path first
        candidates.push(clean_path.to_string());
        
        // Strategy 2: If path doesn't end with .html, try adding .html
        if !clean_path.ends_with(".html") && !clean_path.contains('.') {
            candidates.push(format!("{}.html", clean_path));
        }
        
        // Strategy 3: Try as directory with index.html
        candidates.push(format!("{}/index.html", clean_path));
        
        candidates
    }

    /// Returns 404.html as fallback, or creates a basic 404 response if 404.html doesn't exist
    fn get_404_fallback() -> Result<(PathBuf, String), Error> {
        let fallback_path = Path::new("./src/public/404.html");
        if fallback_path.exists() {
            Ok((fallback_path.to_path_buf(), get_mime_type("404.html")))
        } else {
            // Create a basic 404 response if 404.html doesn't exist
            Err(Error::new(
                std::io::ErrorKind::NotFound,
                "File not found and no 404.html available"
            ))
        }
    }

    pub fn get_file(url: &str) -> Result<(Vec<u8>, String), Error> {
        let (file_path, mime_type) = Self::resolve_file_path(url)?;
        let data = fs::read(&file_path)?;
        Ok((data, mime_type))
    }

    pub fn set_body(&mut self, body: Vec<u8>, mime: &str) {
        // println!("[http_response] set_body {} bytes", body.len());
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

    pub fn set_version(&mut self, version: HttpVersion) {
        self.headers.set_version(version)
    }

    /**
        Start an event stream. This is a special type of response that allows the server to send
        multiple responses to the client. This is useful for real-time applications like chat
        applications.
    */
    pub fn start_event_stream(&mut self) -> Result<Flag, Error> {
        self.set_status(HttpStatus::OK);
        self.headers.set("Content-Type", "text/event-stream");
        self.headers.set("Cache-Control", "no-cache");
        self.headers.set("Connection", "keep-alive");
        self.headers.set("X-Accel-Buffering", "no");
        self.body = Some("data: connected!\n\n".to_string().into_bytes());
        Ok(Flag::EventStream)
    }

    /**
        Prepare the response to be sent to the client. The request has 3 parts:
            1. The status line
            2. The headers
            3. The body
    */
    pub fn prepare(&mut self) -> Vec<u8> {
        let mut response = String::new();
        response.push_str(&self.response_headers());
        let mut bytes = response.into_bytes();
        match &self.body {
            Some(body) => {
                bytes.extend(body);
            }
            None => {}
        }
        bytes
    }

    pub fn response_headers(&self) -> String {
        let version = self.headers.version_string();
        let code = self.status.code();
        let message = self.status.message();
        let mut http_response_headers = format!("{} {} {}\r\n", version, code, message);
        for (key, value) in self.headers.raw.borrow() {
            http_response_headers.push_str(&format!("{}: {}\r\n", key, value));
        }
        http_response_headers.push_str("\r\n");
        http_response_headers
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
}
