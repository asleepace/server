use crate::core::http::HttpRequest;
use crate::core::state::SharedState;
use crate::core::traits::ArcRwLock;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::usize;
use std::vec::Vec;

use super::http::http_request;

pub struct Connections {
    tcp_ttl: u32,
    tcp_incoming: SharedState<Vec<Arc<TcpStream>>>,
}

impl Connections {
    pub fn new() -> Self {
        Connections {
            tcp_ttl: 30,
            tcp_incoming: SharedState::new(Vec::new()),
        }
    }

    /// Process the incoming TCP stream by adding the stream to the incoming TCP streams
    /// and returning the current index of the incoming TCP streams. This index will be
    /// used to get the incoming TCP stream and handle the incoming HTTP request.
    pub fn process(&self, stream: TcpStream) -> Result<Arc<TcpStream>, std::io::Error> {
        match self.add(stream) {
            Err(e) => {
                println!("[connections] error processing: {:?}", e);
                Err(e)
            }
            Ok(index) => {
                let tcp_stream = self.get(index)?;
                Ok(tcp_stream)
            }
        }
    }

    /// Handle the incoming TCP stream by setting the TTL, checking if the stream is
    /// valid, and pushing the stream to the incoming TCP streams. The result returned
    /// is the current index of the incoming TCP streams.
    pub fn add(&self, stream: TcpStream) -> Result<usize, std::io::Error> {
        // println!("[connections] incoming: {:?}", stream.peer_addr());
        // stream.set_ttl(self.tcp_ttl)?; // NOTE: will throw!
        // if let Err(e) = stream.set_ttl(self.tcp_ttl) {
        //     println!("[connections] error setting TTL: {:?}", e);
        // }
        // if let Err(e) = stream.set_nonblocking(true) {
        //     println!("[connections] error setting nodelay: {:?}", e);
        // }
        // stream.set_read_timeout(duration)
        // stream.set_write_timeout(duration)
        // stream.set_nonblocking(true)
        self.tcp_incoming.write(|tcp_incoming| {
            tcp_incoming.push(Arc::new(stream));
            println!("[connections] added new stream: {}", tcp_incoming.len());
            Ok(tcp_incoming.len() - 1)
        })
    }

    /// Get the incoming TCP stream by the index. If the index is not found, return
    /// an error. This index will usually be used to get the incoming TCP stream and
    /// handle the incoming HTTP request.
    pub fn get(&self, tcp_stream_index: usize) -> Result<Arc<TcpStream>, std::io::Error> {
        self.tcp_incoming.read(|tcp_incoming| {
            if tcp_stream_index < tcp_incoming.len() {
                Ok(tcp_incoming[tcp_stream_index].clone())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "TCP stream not found",
                ))
            }
        })
    }
}
