use core::cli;
use core::cli::args;
use core::http::http_headers::HttpMethod;
use core::http::HttpRequest;
use core::server::Server;
use core::util::Rand;
use core::ServerEvent;
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

    // Create a new session route: GET /session/new -> 302 redirect to /s/{id}
    server.route("/session/new", |sr| {
        let mut rnd = Rand::new();
        let id = {
            const ALPHANUM: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            let mut s = String::new();
            for _ in 0..6 {
                let n = (rnd.generate_u64() % (ALPHANUM.len() as u64)) as usize;
                s.push(ALPHANUM[n] as char);
            }
            s
        };
        let html = format!("<html><head><meta http-equiv=\"refresh\" content=\"0; url=/s/{}\"/></head><body>redirecting...</body></html>", id);
        match sr.send_text(core::http::HttpStatus::OK, "text/html; charset=utf-8", &html) {
            Ok(_) => Ok(200),
            Err(e) => Err(e),
        }
    });

    // Session page and POST ingest: GET /s/[id] -> html; POST /s/[id] -> broadcast body
    server.route("/s/[id]", |sr| {
        match sr.headers.method {
            HttpMethod::GET => match sr.send_file("session.html") {
                Ok(_) => Ok(200),
                Err(e) => Err(e),
            },
            HttpMethod::POST => {
                // Read body based on Content-Length if provided; otherwise take nothing
                let len = sr
                    .headers
                    .get("Content-Length")
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(0);
                let mut data = String::new();
                if len > 0 {
                    if let Some(conn) = &sr.connection {
                        use std::io::{ErrorKind, Read};
                        use std::time::Duration;
                        let mut s = conn.as_ref();
                        let mut buf = vec![0u8; len];
                        let mut read = 0usize;
                        // Read loop to handle partial reads / WouldBlock
                        while read < len {
                            match s.read(&mut buf[read..]) {
                                Ok(0) => break,
                                Ok(n) => read += n,
                                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                    std::thread::sleep(Duration::from_millis(1));
                                    continue;
                                }
                                Err(_) => break,
                            }
                        }
                        data = String::from_utf8_lossy(&buf[..read]).to_string();
                    }
                }
                // If no Content-Length, attempt to read until socket would block once
                if len == 0 {
                    if let Some(conn) = &sr.connection {
                        use std::io::{ErrorKind, Read};
                        let mut s = conn.as_ref();
                        let mut buf = [0u8; 4096];
                        match s.read(&mut buf) {
                            Ok(n) if n > 0 => {
                                data = String::from_utf8_lossy(&buf[..n]).to_string();
                            }
                            Ok(_) => {}
                            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                            Err(_) => {}
                        }
                    }
                }

                // emit to the specific session if id exists
                if let Some(id) = sr.param("id") {
                    crate::core::http::http_connections::send_event_to_session_global(
                        id,
                        ServerEvent::event("base64", data),
                    );
                }

                match sr.send_text(
                    core::http::HttpStatus::OK,
                    "text/plain; charset=utf-8",
                    "ok",
                ) {
                    Ok(_) => Ok(200),
                    Err(e) => Err(e),
                }
            }
            _ => match sr.send_text(
                core::http::HttpStatus::BadRequest,
                "text/plain",
                "bad request",
            ) {
                Ok(_) => Ok(400),
                Err(e) => Err(e),
            },
        }
    });

    // special endpoint for event-streams
    server.route("/events", |sr| {
        println!("[main] serving route: /events");
        match sr.event_source() {
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
        if let Some(user_id) = sr.param("userId") {
            println!("userId = {}", user_id);
        }
        match sr.send_file("user.html") {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    server.route("/posts/[postId]", |sr| {
        println!("[main] serving dynamic route: /posts/[postId]");
        if let Some(id) = sr.param("postId") {
            println!("postId = {}", id);
        }
        match sr.send_file("post.html") {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    server.route("/posts/[postId]/comments/[commentId]", |sr| {
        println!("[main] serving dynamic route: /posts/[postId]/comments/[commentId]");
        if let Some(pid) = sr.param("postId") {
            println!("postId = {}", pid);
        }
        if let Some(cid) = sr.param("commentId") {
            println!("commentId = {}", cid);
        }
        match sr.send_file("comment.html") {
            Ok(_) => Ok(200),
            Err(err) => Err(err),
        }
    });

    server.start();
    Ok(())
}
