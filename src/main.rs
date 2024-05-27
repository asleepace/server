use crate::core::server::server::Server;

mod core;

fn main() {
    let mut server = Server::new("localhost", 8080);

    server.route("/", |_| {
        // sr.send_file("index.html");
    });

    server.start();
}
