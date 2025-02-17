use core::cli;
use core::cli::args;
use core::http::HttpRequest;
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

    // MARK: Middleware

    server.middleware(|req, next| {
        println!("[middleware][0] {:?}: {}", req.headers.method, req.uri);
        let time_start = std::time::Instant::now();
        // handle before requests here...
        let res = next(req);
        // handle after requests here...
        let time_end = std::time::Instant::now();
        let duration = time_end - time_start;
        println!(
            "[middleware][0] finished ({:?}): {} ({}ms)",
            req.response.status,
            req.uri,
            duration.as_millis()
        );

        match res {
            Ok(404) => {
                println!("[middleware][0] not found!");
                let _ = req.send_404();
                return Ok(404);
            }
            Ok(401) => {
                println!("[middleware][0] unauthorized access!");
                return Ok(401);
            }
            _ => {}
        }

        return res;
    });

    server.middleware(|req, next| {
        println!("[middleware][1] auth!");

        if req.uri == "/auth" {
            return Ok(401);
        }

        next(req)
    });

    server.middleware(|req, next| {
        let res = next(req);
        // handle after requests here...
        println!(
            "[middleware][2] finished ({:?}): {}",
            req.response.status, req.uri
        );
        res
    });

    // MARK: Routes

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
