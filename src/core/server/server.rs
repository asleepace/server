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

    /**
        Register a route handler.
    */
    pub fn route<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) + 'static,
    {
        self.routes.insert(path.to_string(), Box::new(handler));
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

// fn handle_stream(&self, stream: &TcpStream) {
//     let request = Request::init(stream);
//     // req.info();
//     let url = req.url().unwrap_or("index.html");
//     println!("[server] handle request: {}", url);
//     let cnf = Config::new(self.config.host.as_str(), self.config.port);
//     let mut ctx = Context::new(cnf, &req, res);
//     println!("[server] locating file: {:?}", &url);
//     println!("[server] routes: {:?}", self.routes.keys());
//     println!("[  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  ]");
//     let path = url;
//     let file_path = format!("./src/public/{}", path.trim_matches('/')).to_string();
//     println!("file path: {}", file_path);

//     match self.routes.get(path) {
//         Some(handler) => handler(ctx.borrow_mut()),
//         None => {
//             eprintln!("[server] route not found: {}", path);
//             let sent_asset = ctx.response.file(path);
//             if sent_asset.is_ok() {
//                 ctx.response.status(200);
//                 let _ = ctx.response.send();
//             } else if sent_asset.is_err() {
//                 eprintln!("[server] file not found: {:?}", sent_asset.err());
//                 ctx.response.content_type(ContentType::HTML);
//                 let _ = ctx.response.file("./src/public/404.html");
//                 ctx.response.status(404);
//                 let _ = ctx.response.send();
//             }
//         }
//     }
// }

//     match self.routes.get(path) {
//         Some(handler) => handler(ctx.borrow_mut()),
//         None => match std::fs::read_to_string(file_path) {
//             Ok(content) => {
//                 println!("[server] serving file: {}", path);
//                 if path.ends_with(".css") {
//                     ctx.response.content_type(ContentType::CSS);
//                 } else if path.ends_with(".png") {
//                     ctx.response.content_type(ContentType::PNG);
//                 } else if path.ends_with(".json") {
//                     ctx.response.content_type(ContentType::JSON);
//                 } else {
//                     ctx.response.content_type(ContentType::HTML);
//                 }
//                 ctx.response.body(content);
//                 ctx.response.status(200);
//                 let _ = ctx.response.send();
//             }
//             Err(err) => {
//                 println!("[server] file not found: {:?}", err);
//                 ctx.response.content_type(ContentType::HTML);
//                 ctx.response.file("./src/public/404.html");
//                 ctx.response.status(404);
//                 let _ = ctx.response.send();
//             }
//         },
//     }
// }

// fn catch_all(&self, stream: &TcpStream) {
//     // let non_blocking = stream.set_nonblocking(true);
//     // if non_blocking.is_err() {
//     //     eprintln!("[server] error: unable to set non-blocking mode");
//     //     return;
//     // }

//     println!("[server] stream: {:?}", stream);
//     let request = Request::init(stream);
//     request.info();

//     let mut response = Response::new(stream.try_clone().unwrap());
//     response.content_type(ContentType::HTML);
//     response.header("X-Server-Version", "0.0.1");
//     response.body("Hello, world!".to_string());
//     response.status(200);
//     let output = response.send();
//     println!("[server] response: {:?}", output);
// }
