use crate::core::http::HttpRequest;
use crate::core::middleware::{Middleware, NextHandler, NextResult};
use crate::core::http::http_response::HttpResponse;
use std::io::Error;

/// Static file serving middleware that only triggers if no response has been sent
pub struct StaticFileMiddleware;

impl StaticFileMiddleware {
    pub fn new() -> Self {
        StaticFileMiddleware
    }
}

impl Middleware for StaticFileMiddleware {
    fn handle(&self, request: &mut HttpRequest, next: NextHandler) -> NextResult {
        // Only serve static files if no response has been sent yet
        if request.is_response_sent() {
            return next(request);
        }

        // Try to serve a static file
        match request.serve_static_file() {
            Ok(_) => {
                request.mark_response_sent(200);
                Ok(200)
            }
            Err(_) => {
                // If static file serving fails, continue to next middleware
                next(request)
            }
        }
    }
}

/// Convenience function to create a static file middleware
pub fn static_file_middleware() -> Box<dyn Middleware> {
    Box::new(StaticFileMiddleware::new())
}
