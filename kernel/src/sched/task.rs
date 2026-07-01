//! Task representation for Phoenix OS.

/// The possible states of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is ready to run.
    Ready,
    /// Task is currently running.
    Running,
    /// Task is blocked waiting for an event.
    Blocked,
}

/// A task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub u64);

/// Represents a unit of work (task/thread) in the kernel.
pub struct Task {
    /// The unique identifier for the task.
    pub id: TaskId,
    /// The current state of the task.
    pub state: TaskState,
}

impl Task {
    /// Create a new task.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self {
            id: TaskId(id),
            state: TaskState::Ready,
        }
    }
}
