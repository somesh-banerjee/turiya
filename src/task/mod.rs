use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};

pub mod simple_executor;
pub mod keyboard;
pub mod executor;

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(cx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

use core::sync::atomic::{AtomicU64, Ordering};

impl TaskId {
    fn new() -> TaskId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        // AtomicU64::new(0) creates a new atomic u64 variable with an initial value of 0
        // AtomicU64 is a wrapper around a u64 value that ensures atomic operations
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
        // fetch_add(1, Ordering::Relaxed) atomically increments the value by 1 and returns the previous value
    }
}