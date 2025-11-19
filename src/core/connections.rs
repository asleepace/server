use crate::core::http::HttpRequest;
use crate::core::state::SharedState;
use crate::core::traits::ArcRwLock;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;
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

    /// Configure the incoming TCP stream (Linux/macOS):
    /// - set TTL
    /// - enable TCP_NODELAY
    /// - set read/write timeouts to mitigate slowloris
    /// - keep socket in blocking mode for simple buffered reads
    fn configure(tcp_stream: TcpStream) -> Result<TcpStream, std::io::Error> {
        let _ = tcp_stream.set_ttl(30);
        let _ = tcp_stream.set_nodelay(true);
        let _ = tcp_stream.set_nonblocking(false);
        let _ = tcp_stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = tcp_stream.set_write_timeout(Some(Duration::from_secs(5)));
        Ok(tcp_stream)
    }

    /// Process the incoming TCP stream by adding the stream to the incoming TCP streams
    /// and returning the current index of the incoming TCP streams. This index will be
    /// used to get the incoming TCP stream and handle the incoming HTTP request.
    pub fn process(&self, stream: TcpStream) -> Result<Arc<TcpStream>, std::io::Error> {
        // Configure and wrap without retaining in shared state to avoid growth
        let stream = match Connections::configure(stream) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };
        Ok(Arc::new(stream))
    }

    /// Handle the incoming TCP stream by setting the TTL, checking if the stream is
    /// valid, and pushing the stream to the incoming TCP streams. The result returned
    /// is the current index of the incoming TCP streams.
    pub fn add(&self, stream: TcpStream) -> Result<usize, std::io::Error> {
        let stream = Connections::configure(stream)?;
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
