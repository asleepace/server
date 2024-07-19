use core::cli;
use core::server::Server;

mod core;

fn main() {
    let _argv = cli::process_args();

    let mut server = Server::new("localhost", 8080);

    server.route("/", |sr| {
        sr.response.append("<h1>Hello, World!</h1>");
    });

    server.start();
}
