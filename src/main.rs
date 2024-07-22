use core::cli;
use core::cli::args;
use core::http3::quic::{Connection, QuicError};
use core::http3::{walker, Http3Connection, Http3Server};
use core::server::Server;
use core::Stdout;
use std::future::Future;
use std::io::Error;
use std::net::UdpSocket;
use std::task::Poll;
use std::thread;

mod core;

fn main() {
    // Process command line arguments.
    let argv = cli::process_args();

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

    server.route("/log", |sr| {
        println!("[main] serving route: events.html");
        sr.send_file("log.html")
    });

    // special endpoint for event-streams
    server.route("/events", |sr| {
        println!("[main] serving route: events.html");
        sr.event_souce()
    });

    server.route("/info", |sr| {
        println!("[main] serving route: info.html");
        sr.send_file("info.html")
    });

    server.start();
}
