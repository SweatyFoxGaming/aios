//! Process scheduler for Phoenix OS.

pub mod task;
pub mod process;

use crate::sched::task::Task;
use lazy_static::lazy_static;
use spin::Mutex;

/// Maximum number of concurrent tasks.
const MAX_TASKS: usize = 64;

lazy_static! {
    /// Global scheduler instance.
    pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

/// A simple cooperative scheduler.
pub struct Scheduler {
    tasks: [Option<Task>; MAX_TASKS],
    /// Index of the currently running task.
    pub current_task_idx: usize,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    /// Create a new scheduler.
    #[must_use]
    pub const fn new() -> Self {
        // Workaround for non-const array initialization
        const INIT: Option<Task> = None;
        Self {
            tasks: [INIT; MAX_TASKS],
            current_task_idx: 0,
        }
    }

    /// Add a new task to the scheduler.
    pub fn add_task(&mut self, task: Task) -> bool {
        for slot in &mut self.tasks {
            if slot.is_none() {
                *slot = Some(task);
                return true;
            }
        }
        false
    }

    /// Run the next task (Preemptive context switch).
    pub fn schedule(&mut self) {
        // Simple round-robin
        self.current_task_idx = (self.current_task_idx + 1) % MAX_TASKS;
        // In a real implementation, we would perform a context switch here
    }

    /// List all tasks.
    pub fn list_tasks(&self) {
        crate::println!("--- Active Tasks ---");
        for task in self.tasks.iter().flatten() {
            crate::println!("Task ID: {:?}, State: {:?}", task.id, task.state);
        }
        crate::println!("Current Task Index: {}", self.current_task_idx);
        crate::println!("--------------------");
    }
}

/// Initialize the scheduler.
pub fn init() {
    let mut sched = SCHEDULER.lock();
    let kernel_task = Task::new(0);
    let _ = sched.add_task(kernel_task);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn scheduler_round_robin_advances_index() {
        let mut sched = SCHEDULER.lock();
        let old = sched.current_task_idx;
        sched.schedule();
        assert_eq!(sched.current_task_idx, (old + 1) % MAX_TASKS);
    }

    #[test_case]
    fn scheduler_add_task_succeeds_until_full() {
        // A fresh local Scheduler, not the shared global -- avoids
        // disturbing boot state the rest of the kernel depends on.
        let mut sched = Scheduler::new();
        for i in 0..MAX_TASKS {
            assert!(sched.add_task(Task::new(i as u64)));
        }
        assert!(!sched.add_task(Task::new(999)));
    }
}
