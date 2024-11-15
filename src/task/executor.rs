// Import necessary types and modules
use super::{Task, TaskId}; // `Task` and `TaskId` are used for managing individual tasks
use alloc::{collections::BTreeMap, sync::Arc}; // `BTreeMap` for task storage, `Arc` for thread-safe shared ownership
use core::task::{Waker, Context, Poll}; // Core types for async task management
use crossbeam_queue::ArrayQueue; // Lock-free queue for task scheduling

/// The `Executor` struct is responsible for managing and running asynchronous tasks.
/// It maintains a task queue, tracks tasks, and uses wakers for efficient task scheduling.
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>, // Store all tasks by their ID for quick access
    task_queue: Arc<ArrayQueue<TaskId>>, // Queue of ready-to-run task IDs
    waker_cache: BTreeMap<TaskId, Waker>, // Cache wakers to avoid recreating them
}

impl Executor {
    /// Create a new `Executor` instance.
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)), // Supports up to 100 tasks
            waker_cache: BTreeMap::new(),
        }
    }

    /// Add a new task to the executor.
    /// - Assigns the task to the task map using its unique ID.
    /// - Pushes the task ID into the task queue for execution.
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("Task with the same ID already exists in the executor");
        }
        self.task_queue.push(task_id).expect("Task queue is full");
    }

    /// Execute all tasks that are ready to run.
    /// - Polls each task in the queue, checking if it is ready or still pending.
    pub fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        // Loop through all tasks in the queue
        while let Some(task_id) = task_queue.pop() {
            // Retrieve the task from the map
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // Skip if the task is not found (e.g., already completed)
            };

            // Get or create a waker for the task
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            // Create a `Context` for the task using the waker
            let mut context = Context::from_waker(waker);

            // Poll the task to see if it's ready or still pending
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // If the task is complete, remove it from the task map and waker cache
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {} // If still pending, leave it in the map
            }
        }
    }

    /// Continuously run the executor until all tasks are completed.
    /// - Executes ready tasks and enters a low-power state if idle.
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks(); // Run all tasks that are ready
            self.sleep_if_idle();  // Enter sleep mode if no tasks are ready
        }
    }

    /// Sleep when idle to save CPU cycles.
    /// - Uses `hlt` instruction (halt CPU) when there are no tasks in the queue.
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable(); // Disable interrupts temporarily
        if self.task_queue.is_empty() {
            enable_and_hlt(); // Enable interrupts and halt the CPU
        } else {
            interrupts::enable(); // Re-enable interrupts
        }
    }
}

/// A `TaskWaker` represents a waker tied to a specific task.
/// - Allows the executor to wake up and re-schedule tasks.
struct TaskWaker {
    task_id: TaskId, // ID of the task associated with the waker
    task_queue: Arc<ArrayQueue<TaskId>>, // Shared queue for task scheduling
}

impl TaskWaker {
    /// Wake up the associated task by pushing its ID back into the task queue.
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("Task queue is full");
    }

    /// Create a new `Waker` for the given task.
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker { 
            task_id, 
            task_queue 
        }))
    }
}

// Implement the `Wake` trait for `TaskWaker` to integrate with the async runtime.
use alloc::task::Wake;

impl Wake for TaskWaker {
    /// Wake the task by value (consumes the waker).
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    /// Wake the task by reference (does not consume the waker).
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
