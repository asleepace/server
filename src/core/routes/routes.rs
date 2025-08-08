use crate::core::http::HttpRequest;
use crate::core::middleware::NextResult;
use crate::core::state::SharedState;
use crate::core::traits::ArcRwLock;
use crate::core::routes::RouteMatcher;
use std::collections::HashMap;
use std::sync::Arc;

pub type RouteHandler = Arc<dyn Fn(&mut HttpRequest) -> NextResult + Sync + Send + 'static>;

/// Routes is a collection of route handlers with dynamic path matching support.
/// This is used to register and lookup route handlers by path patterns.
/// The route handlers are stored in a HashMap, where the key is the handler name and the value is the handler.
/// The handler is a function that takes a mutable reference to an `HttpRequest` and returns a `NextResult`.
/// The `NextResult` is an alias for a `Result<u16, std::io::Error>`, where the u16 is the status code of the response.
#[derive(Clone)]
pub struct Routes {
    routes: SharedState<HashMap<String, RouteHandler>>,
    matcher: SharedState<RouteMatcher>,
}

impl Routes {
    /// Create a new Routes instance.
    pub fn new() -> Self {
        Routes {
            routes: SharedState::new(HashMap::new()),
            matcher: SharedState::new(RouteMatcher::new()),
        }
    }

    /// Get a route handler by path pattern matching.
    pub fn get(&self, path: &str) -> Option<(RouteHandler, HashMap<String, String>)> {
        let route_match = self.matcher.read(|matcher| matcher.match_path(path));
        
        if route_match.is_match {
            self.routes.read(|routes| {
                routes.get(&route_match.handler_key).map(|handler| {
                    (handler.clone(), route_match.parameters)
                })
            })
        } else {
            None
        }
    }

    /// Check if a route handler exists by path pattern.
    pub fn has(&self, path: &str) -> bool {
        let route_match = self.matcher.read(|matcher| matcher.match_path(path));
        route_match.is_match && self.routes.read(|routes| routes.contains_key(&route_match.handler_key))
    }

    /// Remove a route handler by pattern.
    pub fn remove(&mut self, pattern: &str) {
        let handler_key = self.matcher.read(|matcher| {
            let route_match = matcher.match_path(pattern);
            if route_match.is_match {
                Some(route_match.handler_key)
            } else {
                None
            }
        });
        
        if let Some(key) = handler_key {
            self.routes.write(|routes| routes.remove(&key));
        }
    }

    /// Add a route handler with pattern matching support.
    pub fn add<F>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) -> NextResult + Sync + Send + 'static,
    {
        // Validate pattern if it contains dynamic parameters
        if pattern.contains('[') && !RouteMatcher::is_valid_pattern(pattern) {
            eprintln!("[routes] Invalid pattern: {}", pattern);
            return;
        }

        let handler_key = format!("handler_{}", pattern.replace('/', "_").replace('[', "").replace(']', ""));
        
        println!("[routes] registering route: {} -> {}", pattern, handler_key);
        
        // Register the pattern with the matcher
        self.matcher.write(|matcher| matcher.register(pattern, &handler_key));
        
        // Register the handler
        self.routes.write(|routes| {
            routes.insert(handler_key, Arc::new(handler));
        });
    }

    /// Add a static route handler (backward compatibility).
    pub fn add_static<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut HttpRequest) -> NextResult + Sync + Send + 'static,
    {
        self.add(path, handler);
    }
}
