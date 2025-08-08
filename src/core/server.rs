use crate::core::cli;
use crate::core::connections::Connections;
use crate::core::http::http_request::ResponseState;
use crate::core::http::{HttpRequest, HttpResponse};
use crate::core::middleware::{Middleware, MiddlewareService};
use crate::core::state::SharedState;
use crate::core::util::get_mime_type;
use crate::core::Config;
use crate::core::ServerEvent;
use crate::core::Stdout;

use crate::core::util::BoundedWorkerPool;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::format;
use std::io::{BufWriter, Error, ErrorKind, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Condvar;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

use super::http::HttpConnections;
use super::middleware::{NextHandler, NextResult};
use super::routes::Routes;
use super::traits::ArcRwLock;

pub enum Flag {
    StaticFile,
    DynamicRoute,
    EventStream,
}

pub struct Server {
    config: Config,
    tcp_connections: Connections,
    tcp_listener: TcpListener,
    stdout: SharedState<Stdout>,
    routes: Routes,
    connections: HttpConnections,
    middlewares: SharedState<MiddlewareService>,
    running: Arc<AtomicBool>,
}

impl Server {
    /// Create a new server instance with a TcpListener and Config.
    /// NOTE: Prefer calling `Server::bind` instead of this method,
    /// or use `Server::instance` to create a server instance.
    pub fn new(tcp_listener: TcpListener, config: Config) -> Self {
        Server {
            config,
            tcp_listener,
            tcp_connections: Connections::new(),
            connections: HttpConnections::new(),
            middlewares: SharedState::new(MiddlewareService::new()),
            stdout: SharedState::new(Stdout::new("./src/data/events.csv", "development")),
            routes: Routes::new(),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Create a new server instance from command line arguments,
    /// or will default to `http://localhost:8080` if no arguments
    /// are provided.
    pub fn instance() -> Result<Server> {
        let argv = cli::process_args();
        let port = match cli::args::parse_as_num(&argv, "--port") {
            Some(port) => port as u16,
            None => 8080,
        };
        let host = match cli::args::parse_as_str(&argv, "--host") {
            Some(host) => host,
            None => "localhost".to_string(),
        };
        // Optional CLI overrides for environment-based config
        if let Some(public_dir) = cli::args::parse_as_str(&argv, "--public-dir") {
            if !public_dir.is_empty() {
                std::env::set_var("PUBLIC_DIR", public_dir);
            }
        }
        if let Some(secs) = cli::args::parse_as_float(&argv, "--sse-heartbeat-secs") {
            if secs > 0.0 && secs < 600.0 {
                std::env::set_var("SSE_HEARTBEAT_SECS", format!("{}", secs));
            }
        }
        if let Some(size) = cli::args::parse_as_num(&argv, "--session-history-size") {
            if size > 0 && size <= 10_000 {
                std::env::set_var("SESSION_HISTORY_SIZE", format!("{}", size));
            }
        }
        Server::bind(&host, port)
    }

    /// Log messages to the server's stdout.
    fn event(&self, _name: &str, data: String) {
        self.connections.send_event(ServerEvent::data(&data));
    }

    /// Log error messages to the server's stdout.
    fn error(&self, _name: &str, data: String) {
        eprintln!("[server] server error: {}", data);
        self.connections.send_event(ServerEvent::data(&data));
    }

    /// Create a new server instance bound to a host and port.
    pub fn bind(host: &str, port: u16) -> Result<Self> {
        println!("[server] binding http://{}:{}/", host, port);
        if host.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "host is empty"));
        }
        if port <= 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "invalid port"));
        }
        let config = Config::new(host, port);
        let domain = config.address();
        let connection = TcpListener::bind(&domain)?;
        let server = Server::new(connection, config);
        // register sessions map pointer for global session events
        server.connections.register_global();
        server.event("server_connected", domain);
        Ok(server)
    }

    /// Terminate all connections and shutdown server.
    pub fn shutdown(&self) {
        self.event("server_shutdown", self.config.address());
        self.middlewares.write(|mid| mid.clear());
        self.connections.close_all();
        println!("[server] shutting down...");
        self.running.store(false, Ordering::SeqCst);
    }

    /// Call this to register the routes as the final middleware, should be called after
    /// all routes have been defined, but before the server is started.
    fn prepare(&mut self) {
        let routes = self.routes.clone();

        // Register route handler middleware
        self.middleware(move |req, next| {
            if req.is_response_sent() {
                return next(req);
            }

            match routes.get(&req.url()) {
                None => {
                    // No route matched, continue to next middleware
                    next(req)
                }
                Some((handler, parameters)) => {
                    // Store parameters in request for handler access
                    req.set_params(parameters);
                    match handler(req) {
                        Ok(status_code) => {
                            req.mark_response_sent(status_code);
                            Ok(status_code)
                        }
                        Err(err) => {
                            let err_msg = err.to_string();
                            req.mark_error(std::io::Error::new(err.kind(), err_msg.clone()));
                            Err(std::io::Error::new(err.kind(), err_msg))
                        }
                    }
                }
            }
        });

        // Register static file middleware as fallback
        self.middleware(|req, next| {
            if req.is_response_sent() {
                return next(req);
            }

            // Try to serve static file
            match req.serve_static_file() {
                Ok(_) => {
                    req.mark_response_sent(200);
                    Ok(200)
                }
                Err(_) => {
                    // If static file serving fails, continue to next middleware
                    next(req)
                }
            }
        });
    }

    /// Start the server and handle incoming connections using a bounded worker pool.
    /// NOTE: This method is blocking.
    pub fn start(&mut self) {
        self.prepare();
        println!("[server] starting server...");

        // Determine worker count: --workers overrides, else available_parallelism
        let argv = cli::process_args();
        let default_workers = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let worker_count: usize = match cli::args::parse_as_num(&argv, "--workers") {
            Some(n) if n > 0 => n as usize,
            _ => default_workers,
        };

        // Queue capacity (defaults to workers * 1024)
        let queue_capacity: usize = match cli::args::parse_as_num(&argv, "--queue-capacity") {
            Some(n) if n > 0 => n as usize,
            _ => worker_count * 1024,
        };

        // Parser limits (optional flags)
        let max_line =
            cli::args::parse_with_bounds(&argv, "--max-request-line", 8 * 1024, 1024, 64 * 1024);
        let max_headers = cli::args::parse_with_bounds(&argv, "--max-headers", 64, 8, 256);
        let max_body = cli::args::parse_with_bounds(
            &argv,
            "--max-body-bytes",
            2 * 1024 * 1024,
            1024,
            64 * 1024 * 1024,
        );
        crate::core::http::http_request::HttpRequest::set_parser_limits(
            max_line,
            max_headers,
            max_body,
        );

        // Use a reusable bounded worker pool abstraction
        let capacity = queue_capacity;
        let pool = BoundedWorkerPool::<TcpStream>::new(worker_count, capacity, Arc::new(|_| {}));

        // Spawn worker threads that pop from pool and dispatch to self.handle
        thread::scope(|scope| {
            for _ in 0..worker_count {
                let pool_ref = &pool;
                // borrow &self within scope
                let server_ref: &Server = &self;
                scope.spawn(move || loop {
                    let stream = pool_ref.pop_blocking();
                    server_ref.handle(stream);
                });
            }

            let _ = self.tcp_listener.set_nonblocking(true);
            while self.running.load(Ordering::SeqCst) {
                match self.tcp_listener.accept() {
                    Ok((stream, _)) => pool.submit(stream),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => {
                        self.error("err_incoming_stream", e.to_string());
                        thread::sleep(Duration::from_millis(25));
                    }
                }
            }
        });
    }

    /// Store the incoming TCP stream and convert it to an HttpRequest.
    /// NOTE: Intermediate method to handle the incoming TCP stream.
    fn tcp_to_http(&self, tcp_stream: TcpStream) -> Result<HttpRequest> {
        match self.tcp_connections.process(tcp_stream) {
            Ok(stream) => HttpRequest::from(stream),
            Err(error) => Err(error),
        }
    }

    /// Handle the incoming TCP stream by converting it to an HttpRequest, then
    /// piping the request through the middleware chain and handling the response.
    /// NOTE: Intermediate method to handle the incoming TCP stream.
    pub fn handle(&self, tcp_stream: TcpStream) {
        // Minimal keep-alive loop: serve multiple requests until read fails or times out
        let mut stream_opt = Some(tcp_stream);
        while let Some(stream) = stream_opt.take() {
            // 1. convert the TcpStream to an HttpRequest
            let mut request = match self.tcp_to_http(stream) {
                Ok(request) => request,
                Err(error) => {
                    eprintln!("[server] error converting tcp to http: {}", error);
                    break;
                }
            };

            // 2. pipe the request through the middleware chain and get the status code
            let status_code = match self.middlewares.read(|mid| (*mid).handle(&mut request)) {
                Err(err) => {
                    eprintln!("[server] middleware error: {}", err);
                    request.mark_error(err);
                    break;
                }
                Ok(code) => code,
            };

            // 3. if request started an SSE stream, handoff to SSE manager and return
            if request.is_event_stream() {
                // Route to session if present
                if let Some(sid) = request.query_param("s") {
                    self.connections.add_session_stream(sid.clone(), request);
                } else {
                    self.handoff_sse(request);
                }
                break;
            }

            // 4. report status; connection remains open for next request
            match request.get_response_state() {
                ResponseState::Handled(status) => {
                    println!("[server] finished with code: {}", status);
                }
                ResponseState::Error(err) => {
                    eprintln!("[server] request error: {}", err);
                    break;
                }
                ResponseState::NotHandled => {
                    println!("[server] request not handled, status code: {}", status_code);
                }
            }

            // re-acquire the underlying TCP stream to continue serving
            stream_opt = match request.connection.as_ref() {
                None => None,
                Some(arc) => match arc.as_ref().try_clone() {
                    Ok(cloned) => Some(cloned),
                    Err(_) => None,
                },
            };
        }
    }

    /// Handoff a live SSE request to the SSE manager so worker thread is freed
    pub fn handoff_sse(&self, req: HttpRequest) {
        println!("[server] handing off SSE stream: {}", req.uri);
        self.connections.add_stream(req);
    }

    /// Register a route handler for a static file.
    pub fn route<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) -> NextResult + Sync + Send + 'static,
    {
        println!("[server] dynamic route: {}", path);
        self.routes.add(path, handler);
    }

    /// Register a middleware handler for the server.
    pub fn middleware<F>(&mut self, handler: F)
    where
        F: Fn(&mut HttpRequest, NextHandler) -> NextResult + Send + Sync + 'static,
    {
        self.middlewares.write(|mid| {
            // Create wrapper struct for the closure
            struct ClosureMiddleware<F>(F);

            impl<F> Middleware for ClosureMiddleware<F>
            where
                F: Fn(&mut HttpRequest, NextHandler) -> NextResult + Send + Sync + 'static,
            {
                fn handle<'a>(
                    &'a self,
                    request: &mut HttpRequest,
                    next: Box<dyn FnOnce(&mut HttpRequest) -> Result<u16> + 'a>,
                ) -> Result<u16> {
                    (self.0)(request, next)
                }
            }

            // Box the middleware before registering
            let boxed_middleware: Box<dyn Middleware> = Box::new(ClosureMiddleware(handler));
            mid.register(boxed_middleware);
        });
    }
}
