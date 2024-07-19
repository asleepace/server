use core::cli;
use core::cli::Args;
use core::server::Server;

mod core;

fn main() {
    let argv = cli::process_args();

    println!("[serveros] arguments: {:?}", argv);

    let port_flag = "--port".to_owned();
    let host_flag = "--host".to_owned();

    let port = match argv.get(&port_flag) {
        Some(Args::Number(port)) => port.to_owned(),
        Some(Args::Text(port)) => port.parse::<u16>().unwrap_or(8080),
        _ => 8080,
    };

    let host = match argv.get(&host_flag) {
        Some(Args::Text(host)) => host,
        _ => "localhost",
    };

    println!("[serveros] starting server on http://{}:{}/", host, port);

    let mut server = Server::new(host, port);

    server.route("/", |sr| {
        sr.response.append("<h1>Hello, World!</h1>");
    });

    server.start();
}
