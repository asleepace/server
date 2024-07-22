use std::{collections::HashMap, io::Error};

/**
    QPACK header compression.
*/
pub fn compress_headers(_headers: &HashMap<String, String>) -> Vec<u8> {
    Vec::new()
}

/**
    QPACK header decompression.
*/
pub fn decompress_headers(_bytes: &[u8]) -> Result<HashMap<String, String>, Error> {
    Err(Error::new(
        std::io::ErrorKind::Other,
        "Failed to decompress QPACK headers",
    ))
}
