#[derive(Debug, Clone)]

/**
    Common HTTP status codes.

    https://www.w3schools.com/tags/ref_httpmessages.asp
*/
pub enum HttpStatus {
    OK,
    Created,
    Accepted,
    NoContent,
    MovedPermanently,
    MovedTemporarily,
    NotModified,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    Status(u16, String),
}

impl HttpStatus {
    pub fn message(&self) -> &str {
        match self {
            HttpStatus::OK => "OK",
            HttpStatus::Created => "Created",
            HttpStatus::Accepted => "Accepted",
            HttpStatus::NoContent => "No Content",
            HttpStatus::MovedPermanently => "Moved Permanently",
            HttpStatus::MovedTemporarily => "Moved Temporarily",
            HttpStatus::NotModified => "Not Modified",
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::Unauthorized => "Unauthorized",
            HttpStatus::Forbidden => "Forbidden",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::InternalServerError => "Internal Server Error",
            HttpStatus::NotImplemented => "Not Implemented",
            HttpStatus::BadGateway => "Bad Gateway",
            HttpStatus::ServiceUnavailable => "Service Unavailable",
            HttpStatus::Status(_, message) => message,
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            HttpStatus::OK => 200,
            HttpStatus::Created => 201,
            HttpStatus::Accepted => 202,
            HttpStatus::NoContent => 204,
            HttpStatus::MovedPermanently => 301,
            HttpStatus::MovedTemporarily => 302,
            HttpStatus::NotModified => 304,
            HttpStatus::BadRequest => 400,
            HttpStatus::Unauthorized => 401,
            HttpStatus::Forbidden => 403,
            HttpStatus::NotFound => 404,
            HttpStatus::InternalServerError => 500,
            HttpStatus::NotImplemented => 501,
            HttpStatus::BadGateway => 502,
            HttpStatus::ServiceUnavailable => 503,
            HttpStatus::Status(code, _) => *code,
        }
    }
}
