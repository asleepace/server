use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, VecDeque};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::core::ServerEvent;

use super::HttpRequest;
use std::sync::{OnceLock, Weak};

type SessionsMap = Mutex<HashMap<String, Vec<HttpRequest>>>;
type HistoryMap = Mutex<HashMap<String, VecDeque<ServerEvent>>>;
static SESSIONS_PTR: OnceLock<Weak<SessionsMap>> = OnceLock::new();
static HISTORY_PTR: OnceLock<Weak<HistoryMap>> = OnceLock::new();
static HISTORY_SIZE: OnceLock<usize> = OnceLock::new();

impl HttpConnections {
    pub fn register_global(&self) {
        let _ = SESSIONS_PTR.set(Arc::downgrade(&self.sessions));
        let _ = HISTORY_PTR.set(Arc::downgrade(&self.session_history));
        // Cache size for globals
        let size = std::env::var("SESSION_HISTORY_SIZE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v > 0 && *v <= 10_000)
            .unwrap_or(100);
        let _ = HISTORY_SIZE.set(size);
    }
}

pub fn send_event_to_session_global(session: &str, event: ServerEvent) {
    // Record in history first
    if let Some(weak_hist) = HISTORY_PTR.get() {
        if let Some(arc_hist) = weak_hist.upgrade() {
            let mut hist = arc_hist.lock().unwrap();
            let size = HISTORY_SIZE.get().copied().unwrap_or(100);
            let deque = hist
                .entry(session.to_string())
                .or_insert_with(VecDeque::new);
            deque.push_back(event.clone());
            while deque.len() > size {
                deque.pop_front();
            }
        }
    }

    // Broadcast to active streams
    if let Some(weak_sessions) = SESSIONS_PTR.get() {
        if let Some(arc_sessions) = weak_sessions.upgrade() {
            let mut map = arc_sessions.lock().unwrap();
            if let Some(vec) = map.get_mut(session) {
                vec.retain_mut(|stream| match stream.server_side_event(event.clone()) {
                    Ok(_) => true,
                    Err(_) => false,
                });
            }
        }
    }
}

/// Broadcast an event to all active session streams (and record in each session history)
pub fn broadcast_event_to_all_sessions(event: ServerEvent) {
    // Append to history for every known session
    if let Some(weak_hist) = HISTORY_PTR.get() {
        if let Some(arc_hist) = weak_hist.upgrade() {
            let mut hist = arc_hist.lock().unwrap();
            let size = HISTORY_SIZE.get().copied().unwrap_or(100);
            for (_sid, deque) in hist.iter_mut() {
                deque.push_back(event.clone());
                while deque.len() > size {
                    deque.pop_front();
                }
            }
        }
    }

    // Broadcast to all active session streams
    if let Some(weak_sessions) = SESSIONS_PTR.get() {
        if let Some(arc_sessions) = weak_sessions.upgrade() {
            let mut map = arc_sessions.lock().unwrap();
            for (_sid, vec) in map.iter_mut() {
                vec.retain_mut(|stream| match stream.server_side_event(event.clone()) {
                    Ok(_) => true,
                    Err(_) => false,
                });
            }
        }
    }
}

pub struct HttpConnections {
    connections: Arc<Mutex<Vec<HttpRequest>>>,
    sessions: Arc<Mutex<HashMap<String, Vec<HttpRequest>>>>,
    session_history: Arc<HistoryMap>,
    is_active: Arc<Mutex<bool>>,
}

impl HttpConnections {
    pub fn new() -> Self {
        HttpConnections {
            connections: Arc::new(Mutex::new(Vec::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_history: Arc::new(Mutex::new(HashMap::new())),
            is_active: Arc::new(Mutex::new(false)),
        }
    }

    pub fn close_all(&self) {
        println!("[http_connections] closing all connections ...");
        let mut connections = self.connections.lock().unwrap();
        connections.retain_mut(
            |stream| match stream.server_side_event(ServerEvent::close()) {
                Ok(_) => match stream.close() {
                    Ok(_) => false,
                    Err(_) => {
                        println!("[http_connections] dropping connection...");
                        false
                    }
                },
                Err(_) => {
                    println!("[http_connections] dropping connection...");
                    false
                }
            },
        );
    }

    pub fn send_event(&self, event: ServerEvent) {
        let mut connections = self.connections.lock().unwrap();
        connections.retain_mut(|stream| match stream.server_side_event(event.clone()) {
            Ok(_) => true,
            Err(_) => {
                println!("[http_connections] dropping connection...");
                false
            }
        });
    }

    pub fn send_event_to_session(&self, session: &str, event: ServerEvent) {
        // Record to history
        let size = HISTORY_SIZE.get().copied().unwrap_or(100);
        {
            let mut hist = self.session_history.lock().unwrap();
            let deque = hist
                .entry(session.to_string())
                .or_insert_with(VecDeque::new);
            deque.push_back(event.clone());
            while deque.len() > size {
                deque.pop_front();
            }
        }

        // Broadcast to live session streams
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(vec) = sessions.get_mut(session) {
            vec.retain_mut(|stream| match stream.server_side_event(event.clone()) {
                Ok(_) => true,
                Err(_) => false,
            });
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

    pub fn add_session_stream(&self, session: String, mut stream: HttpRequest) {
        // Replay history to this new stream
        {
            let hist = self.session_history.lock().unwrap();
            if let Some(deque) = hist.get(&session) {
                for ev in deque.iter() {
                    let _ = stream.server_side_event(ev.clone());
                }
            }
        }

        // Register stream for future events
        let mut map = self.sessions.lock().unwrap();
        map.entry(session).or_insert_with(Vec::new).push(stream);
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
                println!(
                    "[http_connections] total connections: {}",
                    connections.lock().unwrap().len()
                );
            }
            loop {
                if rx.try_recv().is_ok() {
                    println!("[http_connections] stopping keep alive thread");
                    break;
                }

                let interval_secs: f64 = std::env::var("SSE_HEARTBEAT_SECS")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|v| *v > 0.0 && *v < 600.0)
                    .unwrap_or(15.0);

                if last_keep_alive.elapsed().as_secs_f64() > interval_secs {
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
