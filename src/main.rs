use core::cli;
use core::cli::args;
use core::http::HttpRequest;
use core::server::Server;
use core::Stdout;
use std::io::{Error, Result};

mod core;

fn main() -> Result<()> {
    // MARK: Server

    let mut server = Server::instance()?;

    println!("[main] server started!");

    // MARK: Middleware

    server.middleware(|req, next| {
        println!("{:?}: {}", req.headers.method, req.uri);
        // handle before requests here...
        let res = next(req);
        // handle after requests here...
        match res {
            Ok(404) => {
                let _ = req.send_404();
                return Ok(404);
            }
            Ok(401) => Ok(401),
            _ => res,
        }
    });

    // Example Auth middleware
    server.middleware(|req, next| {
        if req.uri == "/auth" {
            return Ok(401);
        }
        next(req)
    });

    // Example post-processing
    server.middleware(|req, next| {
        let res = next(req);
        // handle after requests here...
        res
    });

    // MARK: Routes

    // Static routes
    server.route("/", |sr| {
        println!("[main] serving route: /");
        match sr.send_file("index.html") {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    server.route("/log", |sr| {
        println!("[main] serving route: /log");
        match sr.send_file("log.html") {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    // special endpoint for event-streams
    server.route("/events", |sr| {
        println!("[main] serving route: /events");
        match sr.event_souce() {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    server.route("/info", |sr| {
        println!("[main] serving route: /info");
        match sr.send_file("info.html") {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    // Dynamic routes with parameters
    server.route("/users/[userId]", |sr| {
        println!("[main] serving dynamic route: /users/[userId]");
        // TODO: Access userId parameter from request
        match sr.send_file("user.html") {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    server.route("/posts/[postId]", |sr| {
        println!("[main] serving dynamic route: /posts/[postId]");
        // TODO: Access postId parameter from request
        match sr.send_file("post.html") {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    server.route("/posts/[postId]/comments/[commentId]", |sr| {
        println!("[main] serving dynamic route: /posts/[postId]/comments/[commentId]");
        // TODO: Access postId and commentId parameters from request
        match sr.send_file("comment.html") {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    server.start();
    Ok(())
}
