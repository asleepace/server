use core::cli;
use core::cli::args;
use core::server::Server;

mod core;

fn main() {
    // Process command line arguments.
    let argv = cli::process_args();

    println!("[serveros] argv: {:?}", argv);

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

    println!("[serveros] http://{}:{}/", host, port);

    // Start the server.
    let mut server = Server::new(&host, port);

    // Define routes.
    server.route("/", |sr| {
        println!("[main] serving route: /");
        sr.send_file("index.html");
    });

    server.route("/info", |sr| {
        println!("[main] serving route: info.html");
        sr.send_file("info.html");
    });

    server.start();
}
