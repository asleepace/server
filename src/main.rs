mod config;
mod headers;
mod response;
mod server;

fn main() {
    server::start(config::server());
}
