use crate::core::http::HttpRequest;
use crate::core::middleware::NextResult;
use std::collections::HashMap;

type RouteHandler = Box<dyn Fn(&mut HttpRequest) -> NextResult + Sync + Send + 'static>;

/// Routes is a collection of route handlers.
/// This is used to register and lookup route handlers by path.
/// The route handlers are stored in a HashMap, where the key is the path and the value is the handler.
/// The handler is a function that takes a mutable reference to an `HttpRequest` and returns a `NextResult`.
/// The `NextResult` is an alias for a `Result<u16, std::io::Error>`, where the u16 is the status code of the response.
pub struct Routes {
    routes: HashMap<String, RouteHandler>,
}

impl Routes {
    /// Create a new Routes instance.
    pub fn new() -> Self {
        Routes {
            routes: HashMap::new(),
        }
    }

    /// Get a route handler by path.
    pub fn get(&self, path: &str) -> Option<&RouteHandler> {
        self.routes.get(path)
    }

    /// Check if a route handler exists by path.
    pub fn has(&self, path: &str) -> bool {
        self.routes.contains_key(path)
    }

    /// Remove a route handler by path.
    pub fn remove(&mut self, path: &str) {
        self.routes.remove(path);
    }

    /// Add a route handler to the collection.
    pub fn add<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) -> NextResult + Sync + Send + 'static,
    {
        println!("[routes] registering route: {}", path);
        self.routes.insert(path.to_string(), Box::new(handler));
    }
}
