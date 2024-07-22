use crate::core::http3::quic;
use std::collections::HashMap;
use std::io::Error;
use std::net::UdpSocket;

/**
    TLS 1.3 handshake over QUIC
*/
pub fn perform_handshake(_quic_conn: &mut quic::Connection) -> Result<(), Error> {
    Ok(())
}
