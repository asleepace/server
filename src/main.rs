use core::cli;
use core::cli::args;
use core::server::Server;

mod core;

fn main() {
    let argv = cli::process_args();

    println!("[serveros] argv: {:?}", argv);

    let port = match args::parse_as_num(&argv, "--port") {
        Some(port) => port,
        None => 8080,
    };

    let host = match args::parse_as_str(&argv, "--host") {
        Some(host) => host,
        None => "localhost".to_string(),
    };

    println!("[serveros] http://{}:{}/", host, port);

    let mut server = Server::new(&host, port);
    server.route("/", |_| println!("[serveros] serving index!"));
    server.start();
}
