use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use server::core::util::BoundedWorkerPool;

#[test]
fn pool_handles_all_items() {
    let handled: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let pool = BoundedWorkerPool::<usize>::new(4, 64, Arc::new(|_| {}));

    // spawn workers
    thread::scope(|s| {
        for _ in 0..4 {
            let pool_ref = &pool;
            let handled_ref = handled.clone();
            s.spawn(move || {
                for _ in 0..250 {
                    let item = pool_ref.pop_blocking();
                    let mut g = handled_ref.lock().unwrap();
                    *g += item;
                }
            });
        }

        // submit 4 * 250 items of value 1
        for _ in 0..1000 {
            pool.submit(1);
        }
    });

    // allow workers to finish
    thread::sleep(Duration::from_millis(50));
    let total = *handled.lock().unwrap();
    assert_eq!(total, 1000);
}

// Smoke test scaffold for SSE: ensure header detection works
#[test]
fn sse_header_detection() {
    use server::core::http::http_request::HttpRequest;
    use server::core::http::http_response::HttpResponse;
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;

    // Create a dummy connected pair
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        let _ = TcpStream::connect(addr).unwrap();
    });
    let (stream, _) = listener.accept().unwrap();
    let req = HttpRequest::new(Arc::new(stream));

    // Not SSE yet
    assert!(!req.is_event_stream());
}
