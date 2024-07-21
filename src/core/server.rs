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
    routes: HashMap<String, Box<dyn Fn(&mut HttpRequest) + 'static>>,
}

pub enum ServerError {
    IoError(std::io::Error),
}

impl Server {
    pub fn new(connection: TcpListener, config: Config) -> Self {
        Server {
            config,
            connection,
            routes: HashMap::new(),
        }
    }

    /**
        Create a new server instance.
    */
    pub fn bind(host: &str, port: u16) -> Result<Self> {
        let config = Config::new(host, port);
        let domain = config.address();
        let connection = TcpListener::bind(&domain)?;
        let server = Server::new(connection, config);
        Ok(server)
    }

    /**
       Start the server and listen for incoming connections.
       This method will block the current thread until the server.
    */
    pub fn start(&mut self) {
        self.config.print();
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

    fn handle_stream(&self, tcp_stream: Arc<TcpStream>) -> Result<()> {
        println!("[server] handling stream: {:?}", tcp_stream);

        let mut request = HttpRequest::with(tcp_stream)?;
        // request.info();

        let url = match request.url() {
            Some(uri) => uri.to_owned(),
            None => {
                println!("[server] error: no url found");
                return Err(Error::new(ErrorKind::InvalidInput, "no url found"));
            }
        };

        match self.routes.get(&url) {
            Some(handler) => Ok(handler(&mut request)),
            None => request.serve_static_file(),
        }
    }

    /**
        Register a route handler.
    */
    pub fn route<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) + 'static,
    {
        println!("[server] dynamic route: {}", path);
        self.routes.insert(path.to_string(), Box::new(handler));
    }
}
