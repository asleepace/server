/**
    Middleware trait for implementing custom request handling logic,
    such as rate limiting, authentication, etc.
*/
pub trait Middleware: Send + Syn + 'static {
    fn handle(&self, request: &mut HttpRequest) -> Result<Flag>;
}
