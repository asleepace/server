use crate::core::http::HttpRequest;
use crate::core::middleware::{Middleware, NextResult};

/**
    IntoMiddleware trait for converting a type into a Middleware.
*/
pub trait MiddlewareAdapter: Send + Sync + 'static {
    fn handle(&self, request: &mut HttpRequest) -> NextResult;
}

impl<T: MiddlewareAdapter> Middleware for T {
    fn handle(
        &self,
        request: &mut HttpRequest,
        next: Box<dyn FnOnce(&mut HttpRequest) -> Result<u16, std::io::Error>>,
    ) -> Result<u16, std::io::Error> {
        println!("[middleware] adapter not implemented!");
        return next(request);
    }
}
