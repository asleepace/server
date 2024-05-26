use crate::config::Config;
use crate::headers::ContentType;
use crate::response::Response;
use std::net::{TcpListener, TcpStream};

pub fn start(config: Config) {
    println!("\n[ + - + - + - + - + - + - + - + - + - + - + - + - + ]\n");
    println!("[server] started!");
    config.print();

    match TcpListener::bind(config.address()) {
        Ok(listener) => accept(&listener),
        Err(e) => eprintln!("[server] error: {}", e),
    }
}

pub fn accept(listener: &TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle(&stream),
            Err(error) => eprintln!("[server] accept error: {}", error),
        }
    }
}

pub fn handle(stream: &TcpStream) {
    println!("[server] stream: {:?}", stream);
    let mut response = Response::new(200);
    response.content_type(ContentType::TEXT);
    response.header("X-Server-Version", "0.0.1");
    response.body("Hello, world!".to_string());
    response.status(200);
    match response.write(stream) {
        Ok(_) => println!("[server] response sent!"),
        Err(e) => eprintln!("[server] response error: {}", e),
    }
}
