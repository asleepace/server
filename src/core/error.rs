use std::io::{Error, ErrorKind};

pub struct ServerError {
    pub message: String,
}

impl ServerError {
    pub fn file_not_found(message: &str) -> Error {
        Error::new(ErrorKind::NotFound, message)
    }

    pub fn failed_to_read_file(message: &str) -> Error {
        Error::new(ErrorKind::InvalidInput, message)
    }

    pub fn error(message: &str) -> Error {
        Error::new(ErrorKind::Other, message)
    }
}
