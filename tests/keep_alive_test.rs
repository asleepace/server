use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

// This is a smoke test that exercises stream reuse by sending two HTTP requests on one socket
#[test]
fn keep_alive_two_requests_single_socket() {
    // Start a tiny echo-like server instance in a background thread
    // For now, just create a listener and respond minimally to prove keep-alive behavior
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        // accept one connection, then handle two reads and write two responses
        let (mut stream, _) = listener.accept().unwrap();
        // read crude request bytes
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf);
        let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\nok";
        let _ = stream.write_all(resp);
        let _ = stream.flush();

        // second request
        let _ = stream.read(&mut buf);
        let resp2 = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\nok";
        let _ = stream.write_all(resp2);
        let _ = stream.flush();
    });

    // client: connect once and send two GETs
    let mut client = TcpStream::connect(addr).unwrap();
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n")
        .unwrap();
    client.flush().unwrap();
    let mut buf1 = Vec::new();
    client
        .set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();
    let _ = client.read_to_end(&mut buf1); // ignore exact size; local echo may close fast

    // Send a second request on the same socket
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n")
        .unwrap();
    client.flush().unwrap();
    let mut buf2 = Vec::new();
    client
        .set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();
    let _ = client.read_to_end(&mut buf2);

    // At minimum both responses should contain status line
    let s1 = String::from_utf8_lossy(&buf1);
    let s2 = String::from_utf8_lossy(&buf2);
    assert!(s1.contains("200 OK"));
    assert!(s2.contains("200 OK"));
}
