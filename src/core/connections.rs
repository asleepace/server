use crate::core::http::HttpRequest;
use crate::core::traits::SharedState;
use std::net::TcpListener;
use std::vec::Vec;

pub enum Network {
    Tcp(TcpListener),
    Http(HttpRequest),
    Sse(HttpRequest),
}

pub struct Connections {
    incoming: SharedState<Vec<Network>>,
    delegate: SharedState<Vec<Network>>,
}

impl Connections {
    pub fn new() -> Self {
        Connections {
            incoming: SharedState::new(Vec::new()),
            delegate: SharedState::new(Vec::new()),
        }
    }

    pub fn start_listener(&self, listener: TcpListener) {
        self.incoming.write(|incoming| {
            incoming.push(Network::Tcp(listener));
        });
    }

    pub fn incoming(&self) -> &SharedState<Vec<Network>> {
        &self.incoming
    }

    pub fn delegate(&self) -> &SharedState<Vec<Network>> {
        &self.delegate
    }
}
