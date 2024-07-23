use std::borrow::BorrowMut;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::{collections::HashMap, net::TcpStream};

use crate::core::ServerEvent;

use super::HttpRequest;

pub struct HttpConnections {
    connections: Arc<Mutex<Vec<HttpRequest>>>,
    is_active: Arc<Mutex<bool>>,
}

impl HttpConnections {
    pub fn new() -> Self {
        HttpConnections {
            connections: Arc::new(Mutex::new(Vec::new())),
            is_active: Arc::new(Mutex::new(false)),
        }
    }

    /**
     * Add a new stream to the connections list and start the keep alive thread if not running.
     * This will allow the server to keep the connection alive for a longer period of time.
     */
    pub fn add_stream(&self, stream: HttpRequest) {
        self.connections.lock().unwrap().push(stream);

        // Start the keep alive thread if it's not running.
        if *self.is_active.lock().unwrap() == false {
            self.start_keep_alive_thread();
        }
    }

    pub fn start_keep_alive_thread(&self) -> mpsc::Sender<()> {
        println!("[http_connections] starting keep alive thread...");
        let connections = Arc::clone(&self.connections);
        let is_active = Arc::clone(&self.is_active);
        let (tx, rx) = mpsc::channel::<()>();
        thread::spawn(move || {
            let mut last_keep_alive = Instant::now();
            {
                let mut is_active = is_active.lock().unwrap();
                *is_active = true;
            }
            loop {
                if rx.try_recv().is_ok() {
                    println!("[http_connections] stopping keep alive thread");
                    break;
                }

                if last_keep_alive.elapsed().as_millis() > 300 {
                    let event = ServerEvent::keep_alive();
                    let mut connections_unlocked = connections.lock().unwrap();
                    let total_connections = connections_unlocked.len();
                    connections_unlocked.retain_mut(|stream| {
                        match stream.server_side_event(event.clone()) {
                            Ok(_) => true,
                            Err(_) => {
                                println!("[http_connections] dropping connection...");
                                false
                            }
                        }
                    });

                    println!(
                        "[http_connections] total connections: {}",
                        total_connections
                    );

                    if total_connections == 0 {
                        println!("[http_connections] no more connections, stopping thread.");
                        let mut is_active = is_active.lock().unwrap();
                        *is_active = false;
                        break;
                    }

                    last_keep_alive = Instant::now();
                }

                thread::sleep(Duration::from_millis(100));
            }
        });

        // Return the sender so the caller can stop the thread.
        tx
    }
}
