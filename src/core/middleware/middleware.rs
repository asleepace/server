use std::f32::consts::PI;

use crate::core::http::HttpRequest;

/**
    Middleware trait for implementing custom request handling logic,
    such as rate limiting, authentication, etc.
*/
pub trait Middleware: Send + Sync + 'static {
    fn handle(
        &self,
        request: &mut HttpRequest,
        next: Box<dyn FnOnce(&mut HttpRequest) -> Result<u16, std::io::Error>>,
    ) -> Result<u16, std::io::Error>;
}

/**
    MiddlewareService is a service for registering and executing middleware
    in a chain of responsibility pattern.
*/
pub struct MiddlewareService {
    chain: Vec<Box<dyn Middleware>>,
}

impl MiddlewareService {
    pub fn new() -> MiddlewareService {
        MiddlewareService { chain: Vec::new() }
    }

    pub fn register(&mut self, middleware: Box<dyn Middleware>) {
        println!("[middleware] registering middleware...");
        self.chain.push(middleware);
    }

    pub fn handle(&mut self, request: &mut HttpRequest) -> Result<u16, std::io::Error> {
        self.exec_middleware_chain(request, 0)
    }

    /**
        Execute the middleware chain in order from first to last, and the propogating back
        down the chain from last to first. This is done via a recursive function call.
    */
    fn exec_middleware_chain(
        &mut self,
        request: &mut HttpRequest,
        index: usize,
    ) -> Result<u16, std::io::Error> {
        if index >= self.chain.len() {
            return Ok(0);
        }
        println!("[middleware] executing middleware #{}", index);

        // Create pointer before the closure
        let this = self as *mut MiddlewareService;

        // Safety: `this` pointer is valid for the duration of middleware execution
        // since MiddlewareService outlives all middleware calls
        self.chain[index].handle(
            request,
            Box::new(move |request: &mut HttpRequest| unsafe {
                (*this).exec_middleware_chain(request, index + 1)
            }),
        )
    }
}
