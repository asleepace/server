use crate::core::cli;
use crate::core::http::{HttpRequest, HttpResponse};
use crate::core::middleware::{Middleware, MiddlewareService};
// use crate::core::traits::SharedState;
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
use super::traits::traits::SharedState;

pub enum Flag {
    StaticFile,
    DynamicRoute,
    EventStream,
}

pub struct Server {
    config: Config,
    // http_connections: SharedState,
    tcp_listener: TcpListener,
    stdout: RefCell<Stdout>,
    routes: Arc<
        Mutex<
            HashMap<String, Box<dyn Fn(&mut HttpRequest) -> Result<Flag> + Sync + Send + 'static>>,
        >,
    >,
    connections: HttpConnections,
    middleware_service: Arc<Mutex<MiddlewareService>>,
}

impl Server {
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

    /// Create a new server instance with a TcpListener and Config.
    /// NOTE: Prefer calling `Server::bind` instead of this method,
    /// or use `Server::instance` to create a server instance.
    pub fn new(tcp_listener: TcpListener, config: Config) -> Self {
        Server {
            config,
            tcp_listener,
            connections: HttpConnections::new(),
            middleware_service: Arc::new(Mutex::new(MiddlewareService::new())),
            stdout: RefCell::new(Stdout::new("./src/data/events.csv", "development")),
            routes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Log messages to the server's stdout.
    fn log(&self, _name: &str, data: String) {
        self.connections.send_event(ServerEvent::data(&data));
    }

    /// Log error messages to the server's stdout.
    fn log_error(&self, _name: &str, data: String) {
        eprintln!("[server] server error: {}", data);
        self.connections.send_event(ServerEvent::data(&data));
    }

    /** Create a new server instance bound to a host and port. */
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
        server.log("server_connected", domain);
        Ok(server)
    }

    /** Terminate all connections and shutdown server. */
    pub fn shutdown(&self) {
        self.log("server_shutdown", self.config.address());
        self.middleware_service.lock().unwrap().clear();
        self.connections.close_all();
        println!("[server] shutting down...");
    }

    /**
        Start the server and handle incoming connections. NOTE: This method is blocking,
        and should be called after all routes have been defined.
    */
    pub fn start(&mut self) {
        // NOTE: This is a bit hacky, but we register the last middleware as a route handler
        // to ensure that the middleware chain is executed before the route handler.
        let routes = Arc::clone(&self.routes);
        self.middleware(
            move |req, next| match routes.lock().unwrap().get(&req.url()) {
                None => {
                    if let Err(err) = req.serve_static_file() {
                        eprintln!("[server] error serving static file: {}", err);
                        let _ = req.send_404();
                    }
                    next(req)
                }
                Some(handler) => {
                    if let Err(err) = handler(req) {
                        eprintln!("[server] error handling route: {}", err);
                        let _ = req.send_404();
                    }
                    Ok(404)
                }
            },
        );

        // register routes as the last middleware
        println!("[server] starting server...");
        for stream in self.tcp_listener.incoming() {
            match stream {
                Err(error) => self.log_error("err_incoming_stream", error.to_string()),
                Ok(stream) => self.handle_request(stream),
            }
        }
    }

    fn handle_request(&self, stream: TcpStream) {
        println!("[server] handle stream: {:?}", stream);
        match HttpRequest::from(Arc::new(stream)) {
            Err(error) => self.log_error("err_http_request", error.to_string()),
            Ok(mut request) => {
                let middleware = Arc::clone(&self.middleware_service);
                let handle = thread::spawn(move || {
                    let mut middleware = middleware.lock().unwrap();
                    match middleware.handle(&mut request) {
                        Err(err) => {
                            println!("[serrver] error on handle: {}", err.to_string());
                        }
                        Ok(_) => {
                            println!("[server] finished handling request");
                        }
                    }
                });

                if let Err(err) = handle.join() {
                    println!("[server] error on handle: {:?}", err);
                }
            }
        }
    }

    /**
        Handle an incoming TcpStream by reading the incoming request and sending a response
        back to the client either from a route handler or by serving a static file.
    */
    // fn handle_stream(&self, tcp_stream: Arc<TcpStream>) -> Result<()> {
    //     println!("+--------------------------------------------------------------------------+");

    //     // TODO: implement rate limiting here

    //     let _peer_addr = tcp_stream.peer_addr()?;
    //     let mut request = HttpRequest::from(tcp_stream)?;
    //     let url = request.url();

    //     // TODO: implement middleware here

    //     self.log("network_request", request.info());

    //     let route_flag = match self.routes.get(&url) {
    //         Some(handler) => handler(&mut request),
    //         None => request.serve_static_file(),
    //     };

    //     let did_handle = match route_flag {
    //         Ok(Flag::StaticFile) => Ok(()),
    //         Ok(Flag::DynamicRoute) => Ok(()),
    //         Ok(Flag::EventStream) => {
    //             println!("[server] adding event stream...");
    //             self.connections.add_stream(request);
    //             return Ok(());
    //         }
    //         Err(err) => {
    //             self.log_error("err_route_flag", err.to_string());
    //             Err(Error::new(ErrorKind::Other, err))
    //         }
    //     };

    //     // debugging
    //     if did_handle.is_err() {
    //         println!("[server] could not handle request: {:?}", url);
    //         self.log_error("err_url_not_handled", url.to_string())
    //     }

    //     // send a 404 if the request was not handled
    //     did_handle.or(request.send_404())
    // }

    /**
        Register a route handler.
    */
    pub fn route<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) -> Result<Flag> + Send + Sync + 'static,
    {
        println!("[server] dynamic route: {}", path);
        match self.routes.lock() {
            Ok(mut routes) => {
                routes.insert(path.to_string(), Box::new(handler));
            }
            Err(err) => {
                eprintln!("[server] failed to register route: {}", err);
            }
        }
    }

    /**
        Register middleware.
    */
    pub fn middleware<F>(&mut self, handler: F)
    where
        F: Fn(&mut HttpRequest, Box<dyn FnOnce(&mut HttpRequest) -> Result<u16>>) -> Result<u16>
            + Send
            + Sync
            + 'static,
    {
        match self.middleware_service.lock() {
            Ok(mut mid) => {
                // Create wrapper struct for the closure
                struct ClosureMiddleware<F>(F);

                impl<F> Middleware for ClosureMiddleware<F>
                where
                    F: Fn(
                            &mut HttpRequest,
                            Box<dyn FnOnce(&mut HttpRequest) -> Result<u16>>,
                        ) -> Result<u16>
                        + Send
                        + Sync
                        + 'static,
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
            }
            Err(err) => {
                eprintln!("[server] failed to register middleware: {}", err);
            }
        }
    }
}
