use core::cli;
use core::cli::args;
use core::http3::quic::{Connection, QuicError};
use core::http3::{walker, Http3Connection, Http3Server};
use core::server::Server;
use std::future::Future;
use std::io::Error;
use std::net::UdpSocket;
use std::task::Poll;

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
pub fn spwan_http3_in_background() {
    let _ = std::thread::spawn(|| loop {
        walker::block_on(async {
            match run_quic_server().await {
                Ok(_) => println!("[main] QUIC server exited successfully"),
                Err(err) => eprintln!("[main] QUIC server exited with error: {:?}", err),
            }
        });

        println!("[main] walker finished running!")
    });
}

fn handle_http3_connection(conn: Http3Connection) {
    // Implement HTTP/3 connection handling
    println!("[main] handle http3 connection: {:?}", conn);
}

/**
    Run a QUIC server on the specified address.
*/
pub async fn run_quic_server() -> Result<(), QuicError> {
    let server = Http3Server::new("127.0.0.1:4433").unwrap();
    println!("[main] QUIC server started on http://127.0.0.1:4433/");

    loop {
        match server.accept().await {
            Ok(connection) => {
                println!("[main] New QUIC connection accepted!");
                handle_http3_connection(connection)
            }
            Err(error) => {
                eprintln!("[main] Failed to accept QUIC connection: {:?}", error);
                return Err(QuicError::ConnectionError);
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
