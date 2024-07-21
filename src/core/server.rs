use crate::core::Config;

use crate::core::http::{HttpRequest, HttpResponse};
use crate::core::util::get_mime_type;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Error, ErrorKind, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub struct Server {
    config: Config,
    connection: TcpListener,
    routes: HashMap<String, Box<dyn Fn(&mut HttpRequest) -> Result<()> + 'static>>,
}

impl Server {
    /**
        Create a new server instance with a TcpListener and Config.
    */
    pub fn new(connection: TcpListener, config: Config) -> Self {
        Server {
            config,
            connection,
            routes: HashMap::new(),
        }
    }

    /**
        Create a new server instance bound to a host and port.
    */
    pub fn bind(host: &str, port: u16) -> Result<Self> {
        println!("[serveros] binding http://{}:{}/", host, port);
        let config = Config::new(host, port);
        let domain = config.address();
        let connection = TcpListener::bind(&domain)?;
        let server = Server::new(connection, config);
        Ok(server)
    }

    /**
        Start the server and handle incoming connections. NOTE: This method is blocking,
        and should be called after all routes have been defined.
    */
    pub fn start(&mut self) {
        for stream in self.connection.incoming() {
            match stream {
                Err(error) => eprintln!("[server] accept error: {}", error),
                Ok(stream) => match self.handle_stream(Arc::new(stream)) {
                    Ok(_) => (),
                    Err(err) => eprintln!("[server] error: {}", err),
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
        let mut request = HttpRequest::from(tcp_stream)?;
        let url = request.url();
        request.info();
        let did_handle = match self.routes.get(&url) {
            Some(handler) => handler(&mut request),
            None => request.serve_static_file(),
        };

        return match did_handle {
            Ok(_) => Ok(()),
            Err(error) => {
                println!("[server] error: {:?}", error);
                request.send_404()
            }
        };
    }

    /**
        Register a route handler.
    */
    pub fn route<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) -> Result<()> + 'static,
    {
        println!("[server] dynamic route: {}", path);
        self.routes.insert(path.to_string(), Box::new(handler));
    }
}
