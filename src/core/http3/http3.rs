use crate::core::http3::{qpack, quic};
use std::collections::HashMap;
use std::io::Error;
use std::net::UdpSocket;

// HTTP/3 Framing
pub mod http3 {
    use std::collections::HashMap;
    use std::io::Error;
    use std::vec::Vec;

    pub enum Frame {
        Data(Vec<u8>),
        Headers(HashMap<String, String>),
        Settings(HashMap<u64, u64>),
    }

    /** http3 frame encoding */
    pub fn encode_frame(_frame: &Frame) -> Vec<u8> {
        Vec::new()
    }

    /** http3 frame decoding */
    pub fn decode_frame(_bytes: &[u8]) -> Result<Frame, Error> {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Failed to decode HTTP/3 frame",
        ))
    }
}

pub struct Http3Server {
    quic_listener: quic::Connection,
}

impl Http3Server {
    fn new(addr: &str) -> Result<Self, Error> {
        let socket = UdpSocket::bind(addr)?;
        let quic_listener = quic::handshake(socket)?;
        Ok(Self { quic_listener })
    }

    /**
        Accept incoming HTTP/3 connection.
        1. Accept incoming QUIC connection.
        2. Perform TLS handshake.
        3. Return a new HTTP/3 connection.
    */
    fn accept(&mut self) -> Result<Http3Connection, Error> {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Failed to accept HTTP/3 connection",
        ))
    }
}

pub struct Http3Connection {
    quic_conn: quic::Connection,
}

impl Http3Connection {
    /**
        Handle incoming HTTP/3 request.
        1. Read and parse HTTP/3 frames.
        2. Process the request.
        3. Generate and send the response.
    */
    fn handle_request(&mut self) -> Result<(), Error> {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Failed to handle HTTP/3 request",
        ))
    }
}
