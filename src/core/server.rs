use crate::core::cli;
use crate::core::connections::Connections;
use crate::core::http::{HttpRequest, HttpResponse};
use crate::core::middleware::{Middleware, MiddlewareService};
use crate::core::state::SharedState;
use crate::core::util::get_mime_type;
use crate::core::Config;
use crate::core::ServerEvent;
use crate::core::Stdout;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::format;
use std::io::{BufWriter, Error, ErrorKind, Result, Write};
use std::net::{TcpListener, TcpStream};
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
        server.event("server_connected", domain);
        Ok(server)
    }

    /// Terminate all connections and shutdown server.
    pub fn shutdown(&self) {
        self.event("server_shutdown", self.config.address());
        self.middlewares.write(|mid| mid.clear());
        self.connections.close_all();
        println!("[server] shutting down...");
    }

    /// Call this to register the routes as the final middleware, should be called after
    /// all routes have been defined, but before the server is started.
    fn prepare(&mut self) {
        let routes = self.routes.clone();
        // NOTE: This is a bit hacky, but we register the last middleware as a route handler
        // to ensure that the middleware chain is executed before the route handler.
        self.middleware(move |req, next| match routes.get(&req.url()) {
            None => req.serve_static_file().and_then(|_| next(req)),
            Some(handler) => handler(req),
        });
    }

    /// Start the server and handle incoming connections.
    /// NOTE: This method is blocking.
    pub fn start(&mut self) {
        self.prepare();
        // register routes as the last middleware
        println!("[server] starting server...");
        for stream in self.tcp_listener.incoming() {
            match stream {
                Err(error) => self.error("err_incoming_stream", error.to_string()),
                Ok(stream) => self.handle(stream),
            }
        }
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
        // 1. convert the TcpStream to an HttpRequest
        let mut request = match self.tcp_to_http(tcp_stream) {
            Ok(request) => request,
            Err(error) => {
                eprintln!("[server] error converting tcp to http: {}", error);
                return;
            }
        };

        // 2. pipe the request through the middleware chain and get the status code
        let status_code = match self.middlewares.write(|mid| (*mid).handle(&mut request)) {
            Err(err) => panic!("[server] error on handle: {}", err.to_string()),
            Ok(code) => code,
        };

        // 3. handle the response based on the status code, then perhaps remove the stream
        // when finished.
        println!("[server] finished with code: {}", status_code);
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
                fn handle(
                    &self,
                    request: &mut HttpRequest,
                    next: Box<dyn FnOnce(&mut HttpRequest) -> Result<u16>>,
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
