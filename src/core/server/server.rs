use crate::core::config::config::Config;

use crate::core::http::http_request::HttpRequest;
use crate::core::http::http_response::HttpResponse;
use crate::core::utils::mime::get_mime_type;

use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};
use std::net::{TcpListener, TcpStream};

pub struct Server {
    config: Config,
    connection: Option<TcpListener>,
    routes: HashMap<String, Box<dyn Fn(&mut HttpRequest) + 'static>>,
}

impl Server {
    /**
        Create a new server instance.
    */
    pub fn new(host: &str, port: u16) -> Self {
        let config = Config::new(host, port);
        let domain = config.address();
        let connection = TcpListener::bind(&domain).ok();
        let routes = HashMap::new();
        Server {
            connection,
            config,
            routes,
        }
    }

    /**
       Start the server and listen for incoming connections.
       This method will block the current thread until the server.
    */
    pub fn start(&self) {
        println!("\n[ + - + - + - + - + - + - + - + - + - + - + - + - + ]\n");
        println!("[server] started!");
        self.config.print();
        match &self.connection {
            Some(listener) => self.accept(&listener),
            None => eprintln!("[server] error: unable to bind to address"),
        }
    }

    fn accept(&self, listener: &TcpListener) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => self.handle_stream(&stream),
                Err(error) => eprintln!("[server] accept error: {}", error),
            }
        }
    }

    fn handle_stream(&self, tcp_stream: &TcpStream) {
        let req = HttpRequest::from(&tcp_stream);
        let mut respone = HttpResponse::new();
        req.info();

        // Step 1: Get the request URL
        let url = match req.url() {
            "/" => "/index.html",
            uri => uri,
        };

        let mime = get_mime_type(url);

        println!("[server] url: {:}", url);
        println!("[server] mime: {:}", mime);

        // Step 2: Locate the file
        let bytes = get_file(url);

        // Step 3: Send the response
        match bytes {
            Ok(data) => {
                let res_data = respone
                    .append(format!("Content-Length: {:}", data.len()).as_str())
                    .append(format!("Content-Type: {:}", mime).as_str())
                    .append_body(&mut data.to_owned());

                let mut writer = BufWriter::new(tcp_stream);
                let did_write = writer.write_all(&mut res_data.to_owned());
                match did_write {
                    Ok(_) => println!("[server] response sent!"),
                    Err(error) => eprintln!("[server] error: {:?}", error),
                }

                writer.flush().unwrap();
                tcp_stream.shutdown(std::net::Shutdown::Both).unwrap();
            }
            Err(error) => {
                eprintln!("[server] error: {:?}", error);
            }
        }
    }

    /**
        Register a route handler.
    */
    pub fn route<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) + 'static,
    {
        self.routes.insert(path.to_string(), Box::new(handler));
    }
}

pub fn get_file(url: &str) -> Result<Vec<u8>, ()> {
    let path = url.trim_matches('/');
    let file_path = format!("./src/public/{}", path);

    // let mut writer = BufWriter::new(&self.stream);
    return match fs::read(file_path.clone()) {
        Ok(data) => Ok(data),
        Err(_) => {
            eprintln!("[response] file not found: {}", file_path);
            return Result::Err(());
        }
    };
}
