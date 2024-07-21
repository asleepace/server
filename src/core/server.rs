use crate::core::Config;

use crate::core::http::{HttpRequest, HttpResponse};
use crate::core::util::get_mime_type;

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
        println!("[server] handling stream: {:?}", tcp_stream);

        let mut request = HttpRequest::from(&tcp_stream);
        let mut response = HttpResponse::new();

        // req.info();

        let request_url = request.url();

        let url = match request_url {
            Some(uri) => uri.to_owned(),
            None => {
                println!("[server] error: no url found");
                return;
            }
        };

        let is_handled = match self.routes.get(&url) {
            Some(handler) => {
                handler(&mut request);
                true
            }
            None => false,
        };

        if is_handled {
            println!("[server] route handled!");
            return;
        }

        // TODO: Improve this in the future.
        let mime = get_mime_type(&url);
        println!("[server] url: {:} ({:})", url, mime);

        // Step 2: Locate the file
        let bytes = get_file(&url);

        // Step 3: Send the response
        match bytes {
            Ok(data) => {
                let res_data = response
                    .append(format!("Content-Length: {:}", data.len()).as_str())
                    .append(format!("Content-Type: {:}", mime).as_str())
                    .append_body(&mut data.to_owned());

                // Step 4: Send the response
                let mut writer = BufWriter::new(tcp_stream);
                let did_write = writer.write_all(&mut res_data.to_owned());
                match did_write {
                    Ok(_) => println!("[server] response sent!"),
                    Err(error) => eprintln!("[server] error: {:?}", error),
                }

                // Step 5: Close the connection
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
        println!("[server] route: {}", path);
        self.routes.insert(path.to_string(), Box::new(handler));
    }
}

/**
    Get the file from the public directory.
*/
pub fn get_file(url: &str) -> Result<Vec<u8>, ()> {
    let path = url.trim_matches('/');
    let file_path = format!("./src/public/{}", path);
    return match fs::read(file_path.clone()) {
        Ok(data) => Ok(data),
        Err(_) => {
            eprintln!("[response] file not found: {}", file_path);
            return Result::Err(());
        }
    };
}
