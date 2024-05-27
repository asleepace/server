use server::Server;

mod config;
mod context;
mod headers;
mod request;
mod response;
mod server;

fn main() {
    let mut server = Server::new("localhost", 8080);

    server.route("/", |sr| {
        sr.send_file("index.html");
    });

    server.start();
}
