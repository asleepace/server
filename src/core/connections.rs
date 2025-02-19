use crate::core::http::HttpRequest;
use crate::core::state::SharedState;
use crate::core::traits::ArcRwLock;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::usize;
use std::vec::Vec;

pub struct Connections {
    tcp_incoming: SharedState<Vec<Arc<TcpStream>>>,
}

impl Connections {
    pub fn new() -> Self {
        Connections {
            tcp_incoming: SharedState::new(Vec::new()),
        }
    }

    /// Configure the incoming TCP stream by setting the TTL and non-blocking options.
    /// NOTE: Some of these options throw errors in dev.
    /// TODO: Handle rate limiting and other options.
    fn configure(tcp_stream: TcpStream) -> Result<TcpStream, std::io::Error> {
        tcp_stream.set_ttl(30)?;
        tcp_stream.set_nonblocking(true)?;
        tcp_stream.set_nodelay(true)?;
        Ok(tcp_stream)
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
        // let stream = Connections::configure(stream)?;
        self.tcp_incoming.write(|tcp_incoming| {
            let arc_stream = Arc::new(stream);
            let idx_stream = tcp_incoming.len();
            tcp_incoming.push(arc_stream);
            println!("[connections] added new stream: {}", idx_stream);
            Ok(idx_stream)
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
