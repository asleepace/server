use core::cli;
use core::cli::args;
use core::http3::quic::{Connection, QuicError};
use core::http3::Http3Server;
use core::server::Server;
use std::io::Error;
use std::net::UdpSocket;

mod core;

fn main() {
    // Process command line arguments.
    let argv = cli::process_args();

    println!("[serveros] argv: {:?}", argv);

    spwan_http3_in_background();

    // Check if the user has specified a port.
    let port = match args::parse_as_num(&argv, "--port") {
        Some(port) => port as u16,
        None => 8080,
    };

    // Check if the user has specified a host.
    let host = match args::parse_as_str(&argv, "--host") {
        Some(host) => host,
        None => "localhost".to_string(),
    };

    // Start the server.
    let mut server = match Server::bind(&host, port) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("[serveros] failed to start server: {}", err);
            return;
        }
    };

    // Define routes.
    server.route("/", |sr| {
        println!("[main] serving route: /");
        sr.send_file("index.html")
    });

    server.route("/info", |sr| {
        println!("[main] serving route: info.html");
        sr.send_file("info.html")
    });

    server.start();
}

/**
    Test function to spawn a HTTP/3 server in the background.
*/
pub fn spwan_http3_in_background() -> std::thread::JoinHandle<()> {
    std::thread::spawn(|| {
        println!("[main] Spawning HTTP/3 server in background...");
        match run_quic_server() {
            Ok(_) => {
                println!("[main] HTTP/3 server started successfully");
            }
            Err(error) => {
                eprintln!("[main] Failed to start HTTP/3 server: {:?}", error);
            }
        }
    })
}

/**
    Run a QUIC server on the specified address.
*/
pub fn run_quic_server() -> Result<(), QuicError> {
    let mut server = Http3Server::new("127.0.0.1:5757").map_err(|_| QuicError::ConnectionError)?;
    println!("[main] QUIC server started on https://localhost:443/");

    loop {
        match server.accept() {
            Ok(mut connection) => {
                println!("[main] Accepted new HTTP/3 connection");
                match connection.handle_request() {
                    Ok(_) => {
                        println!("[main] Handled HTTP/3 request");
                        return Ok(());
                    }
                    Err(_) => {
                        eprintln!("[main] Failed to handle HTTP/3 request");
                        return Err(QuicError::ConnectionError);
                    }
                }
            }
            Err(_) => {
                eprintln!("[main] Failed to accept HTTP/3 connection");
            }
        }
    }

    // let socket = UdpSocket::bind(addr).map_err(|_| QuicError::ConnectionError)?;
    // loop {
    //     let mut buf = [0u8; 65535];
    //     let (_, peer_addr) = socket
    //         .recv_from(&mut buf)
    //         .map_err(|_| QuicError::ConnectionError)?;

    //     let mut connection = Connection::new(socket.try_clone().unwrap(), peer_addr);
    //     connection.handshake()?;
    //     println!("[main] QUIC connection established!");
    //     // Handle connection...
    //     //
    //     connection.open_stream()?;
    // }
}
