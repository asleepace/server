use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    net::TcpStream,
    sync::Arc,
};

/// A fast and lightweight HTTP request parser which will only
/// read the first line of the request to obtain the method, uri, and protocol.
/// The headers and body will be read later when the request is processed.
#[derive(Clone)]
pub struct Request {
    protocol: String,
    method: String,
    uri: String,
    stream: Arc<TcpStream>,
    headers: Option<HashMap<String, String>>,
    body: Option<Vec<u8>>,
}

impl Request {
    /// Maximum size of the peek buffer
    const MAX_PEEK_SIZE: usize = 1024;

    pub fn new(
        protocol: String,
        method: String,
        uri: String,
        stream: Arc<TcpStream>,
        headers: Option<HashMap<String, String>>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Request {
            protocol,
            method,
            uri,
            stream,
            headers,
            body,
        }
    }

    /// Create a new request from a TcpStream.
    /// NOTE: This will attempt to read the first 1024 bytes from the stream
    /// to obtain the request line. The headers and body will be read later
    /// when the request is processed.
    pub fn from(stream: Arc<TcpStream>) -> Result<Self, std::io::Error> {
        let mut buffer = [0_u8; Self::MAX_PEEK_SIZE];
        let n = stream.peek(&mut buffer).map_err(|e| {
            io::Error::new(
                ErrorKind::ConnectionAborted,
                format!("Failed to peek stream: {}", e),
            )
        })?;

        if n == 0 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Empty request"));
        } else {
            println!("[request] read {} / {} bytes", n, Self::MAX_PEEK_SIZE);
        }

        // find the first line ending in the buffer (use memchr for performance?)
        let line_end = buffer[..n]
            .iter()
            .position(|&b| b == b'\n')
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "No line ending found"))?;

        // attempt to read the first request line from the buffer
        // and parse the method, uri, and protocol
        let mut method = None;
        let mut uri = None;
        let mut protocol = None;
        for (i, part) in std::str::from_utf8(&buffer[..line_end])
            .map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e))
            })?
            .trim()
            .split_whitespace()
            .enumerate()
        {
            match i {
                0 => method = Some(part.to_string()),
                1 => uri = Some(part.to_string()),
                2 => protocol = Some(part.to_string()),
                _ => break,
            }
        }

        match (method, uri, protocol) {
            (Some(method), Some(uri), Some(protocol)) => Ok(Request {
                method,
                uri,
                protocol,
                stream,
                headers: None,
                body: None,
            }),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid request line: missing required parts",
            )),
        }
    }

    #[inline]
    pub fn method(&self) -> &str {
        &self.method
    }

    #[inline]
    pub fn uri(&self) -> &str {
        &self.uri
    }

    #[inline]
    pub fn headers(&self) -> Option<&HashMap<String, String>> {
        self.headers.as_ref()
    }

    #[inline]
    pub fn body(&self) -> Option<&Vec<u8>> {
        self.body.as_ref()
    }
}
