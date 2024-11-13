// Import the Task type from the parent module
use super::Task;

// Import VecDeque from alloc for creating a double-ended queue
use alloc::collections::VecDeque;

// Import Context and Poll from core::task for task polling
use core::task::{Context, Poll};

// The SimpleExecutor struct is a minimal async executor that holds a queue of tasks.
pub struct SimpleExecutor {
    // A queue to hold tasks that are waiting to be polled
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    // Creates a new instance of the SimpleExecutor with an empty task queue.
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    // Adds a new task to the end of the task queue.
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    // Runs the executor by continuously polling tasks in the task queue.
    // If a task is not ready (i.e., returns Poll::Pending), it is put back in the queue.
    pub fn run(&mut self) {
        // Continue running as long as there are tasks in the queue
        while let Some(mut task) = self.task_queue.pop_front() {
            // Create a dummy waker for the current task to be polled
            let waker = dummy_waker();
            
            // Create a Context object from the waker to pass into the poll method
            let mut context = Context::from_waker(&waker);

            // Poll the task. If it returns Poll::Pending, re-add it to the end of the queue.
            match task.poll(&mut context) {
                Poll::Pending => self.task_queue.push_back(task),
                Poll::Ready(()) => {} // If the task is complete, do nothing
            }
        }
    }
}

// Import Waker and RawWaker from core::task for creating a dummy waker
use core::task::{Waker, RawWaker};

// Creates a dummy waker that does nothing.
// This is necessary because the executor needs a waker to create a Context.
fn dummy_waker() -> Waker {
    // Waker::from_raw takes a RawWaker, which we create with the dummy_raw_waker function
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

// Import RawWakerVTable for constructing a RawWaker
use core::task::RawWakerVTable;

// Creates a RawWaker with a no-op vtable (a table of function pointers).
fn dummy_raw_waker() -> RawWaker {
    // Define no-op (no operation) functions for the vtable.
    fn no_op(_: *const ()) {} // A function that does nothing (for drop and wake)

    // Define a clone function that creates another dummy_raw_waker.
    // This allows the waker to be safely cloned if needed by the runtime.
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    // Define a RawWakerVTable with our no-op functions
    // This vtable has clone, wake, wake_by_ref, and drop operations
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);

    // Create and return a RawWaker with a null pointer and the vtable
    RawWaker::new(0 as *const (), vtable)
}
