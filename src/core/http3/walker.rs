use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker as TaskWaker};
use std::thread;

struct MyWaker {
    thread: thread::Thread,
}

impl Wake for MyWaker {
    fn wake(self: Arc<Self>) {
        self.thread.unpark();
    }
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut pinned_future = Pin::from(Box::new(future));
    let thread = thread::current();
    let my_waker = Arc::new(MyWaker { thread });
    let waker = TaskWaker::from(my_waker);
    let mut context = Context::from_waker(&waker);

    loop {
        match Future::poll(pinned_future.as_mut(), &mut context) {
            Poll::Ready(val) => break val,
            Poll::Pending => thread::park(),
        }
    }
}
