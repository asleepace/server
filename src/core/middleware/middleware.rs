use crate::core::http::HttpRequest;

/**
    Typealias for the result of a middleware handler, this will return a status code if the
    request has already been handled, otherwise it will be `0`.
*/
pub type NextResult = Result<u16, std::io::Error>;

/**
    The NextHandler type is a function pointer for the next middleware handler in the chain.
*/
pub type NextHandler<'a> = Box<dyn FnOnce(&mut HttpRequest) -> NextResult + 'a>;

/**
    Middleware trait for implementing custom request handling logic,
    such as rate limiting, authentication, etc.
*/
pub trait Middleware: Send + Sync + 'static {
    fn handle<'a>(&'a self, request: &mut HttpRequest, next: NextHandler<'a>) -> NextResult;
}

/**
    MiddlewareService is a service for registering and executing middleware
    in a chain of responsibility pattern.
*/
pub struct MiddlewareService {
    chain: Vec<Box<dyn Middleware>>,
}

/**
    Handles registering middleware and executing the middleware chain, by calling each
    middleware handler in order, and propogating the request back down the chain.
*/
impl MiddlewareService {
    pub fn new() -> MiddlewareService {
        MiddlewareService { chain: Vec::new() }
    }

    pub fn register(&mut self, middleware: Box<dyn Middleware>) {
        println!("[middleware] registering middleware...");
        self.chain.push(middleware);
    }

    pub fn handle(&self, request: &mut HttpRequest) -> Result<u16, std::io::Error> {
        self.exec_middleware_chain(request, 0)
    }

    pub fn clear(&mut self) {
        self.chain.clear();
    }

    /**
        Execute the middleware chain in order from first to last, and the propogating back
        down the chain from last to first. This is done via a recursive function call.
    */
    fn exec_middleware_chain<'a>(&'a self, request: &mut HttpRequest, index: usize) -> NextResult {
        // Reaches the end of the chain and return `0` to indicate that the request was not handled
        if index >= self.chain.len() {
            return Ok(0);
        }

        self.chain[index].handle(
            request,
            Box::new(move |req: &mut HttpRequest| self.exec_middleware_chain(req, index + 1)),
        )
    }
}
