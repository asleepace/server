use crate::core::Config;

use crate::core::http::{HttpRequest, HttpResponse};
use crate::core::util::get_mime_type;
use crate::core::Stdout;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::format;
use std::fs;
use std::io::{BufWriter, Error, ErrorKind, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub enum Flag {
    StaticFile,
    DynamicRoute,
    EventStream,
}

pub struct Server {
    config: Config,
    connection: TcpListener,
    stdout: RefCell<Stdout>,
    routes: HashMap<String, Box<dyn Fn(&mut HttpRequest) -> Result<Flag> + 'static>>,
}

impl Server {
    /**
        Create a new server instance with a TcpListener and Config.
    */
    pub fn new(connection: TcpListener, config: Config) -> Self {
        Server {
            config,
            connection,
            stdout: RefCell::new(Stdout::new("./src/data/events.csv", "development")),
            routes: HashMap::new(),
        }
    }

    /** Log a message to the server's stdout. */
    fn log(&self, name: &str, data: String) {
        let mut writeable = self.stdout.borrow_mut();
        let data_line = format!("{},{}\n", name, data);
        writeable.write(name, data.to_string());
        writeable.send_events(&data_line);
        print!("[server] log: {}", data);
    }

    /** Log error messages to the server's stdout. */
    fn log_error(&self, name: &str, data: String) {
        eprintln!("[server] server error: {}", data);
        let mut writeable = self.stdout.borrow_mut();
        let data_line = format!("{},{}\n", name, data);
        writeable.write(name, data.to_string());
        writeable.send_events(&data_line);
    }

    /** Begin EventSource stream (see stdout) */
    fn begin_event_stream(&mut self, request: HttpRequest) {
        println!("[server] starting event stream...");
        self.stdout.borrow_mut().add_stream(request)
    }

    /** Create a new server instance bound to a host and port. */
    pub fn bind(host: &str, port: u16) -> Result<Self> {
        println!("[serveros] binding http://{}:{}/", host, port);
        let config = Config::new(host, port);
        let domain = config.address();
        let connection = TcpListener::bind(&domain)?;
        let server = Server::new(connection, config);
        server.log("server_connected", domain);
        Ok(server)
    }

    /**
        Start the server and handle incoming connections. NOTE: This method is blocking,
        and should be called after all routes have been defined.
    */
    pub fn start(&mut self) {
        for stream in self.connection.incoming() {
            match stream {
                Err(error) => self.log_error("err_incoming_stream", error.to_string()),
                Ok(stream) => match self.handle_stream(Arc::new(stream)) {
                    Ok(_) => (),
                    Err(err) => {
                        self.log_error("err_server_start", err.to_string());
                        eprintln!("[server] error: {}", err)
                    }
                },
            }
        }
    }

    /**
        Handle an incoming TcpStream by reading the incoming request and sending a response
        back to the client either from a route handler or by serving a static file.
    */
    fn handle_stream(&self, tcp_stream: Arc<TcpStream>) -> Result<()> {
        println!("+--------------------------------------------------------------------------+");
        println!("[server] handling stream: {:?}", tcp_stream);

        self.log("processing", format!("{:?}", tcp_stream));

        let peer_addr = tcp_stream.peer_addr()?;
        let mut request = HttpRequest::from(tcp_stream)?;
        let url = request.url();

        self.log(
            "incoming_stream",
            format!("{}{}", peer_addr.ip().to_string(), url),
        );

        request.info();

        let route_flag = match self.routes.get(&url) {
            Some(handler) => handler(&mut request),
            None => request.serve_static_file(),
        };

        let did_handle = match route_flag {
            Ok(Flag::StaticFile) => Ok(()),
            Ok(Flag::DynamicRoute) => Ok(()),
            Ok(Flag::EventStream) => {
                println!("[server] adding event stream...");
                self.stdout.borrow_mut().add_stream(request);
                return Ok(());
            }
            Err(err) => {
                self.log_error("err_route_flag", err.to_string());
                Err(Error::new(ErrorKind::Other, err))
            }
        };

        // debugging
        if did_handle.is_err() {
            println!("[server] could not handle request: {:?}", url);
            self.log_error("err_url_not_handled", url.to_string())
        }

        // send a 404 if the request was not handled
        did_handle.or(request.send_404())
    }

    /**
        Register a route handler.
    */
    pub fn route<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) -> Result<Flag> + 'static,
    {
        println!("[server] dynamic route: {}", path);
        self.routes.insert(path.to_string(), Box::new(handler));
    }
}
