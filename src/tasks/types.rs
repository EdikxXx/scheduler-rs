use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- State Markers ---
pub struct Pending;
pub struct Waiting;
pub struct Running;
pub struct Success;
pub struct Failed;
pub struct Retrying;
pub struct Canceled;

// --- Base Interface ---
/// The "Blueprint" of your Task in a Distributed System
#[async_trait]
pub trait TaskState: Send + Sync {
    /// Unique Task Data (Must be serializable to cross the Broker boundary)
    type Payload: Send + Sync + Serialize + for<'de> Deserialize<'de>;

    /// Current State Marker (Pending, Running, etc.)
    type State: Send + Sync;

    fn id(&self) -> Uuid;
    fn payload(&self) -> &Self::Payload;
    fn task_name() -> &'static str;
}

// --- Lifecycle Transitions (Async) ---

#[async_trait]
pub trait PendingTask: TaskState {
    type Waiting: WaitingTask;
    type Canceled: CanceledTask;

    /// Sends the task payload to the Message Broker (Network I/O)
    async fn submit(self) -> Self::Waiting;

    /// Flags the task as canceled in the local or remote registry
    async fn cancel(self) -> Self::Canceled;
}

#[async_trait]
pub trait WaitingTask: TaskState {
    type Running: RunningTask;

    /// Fetches the task from the queue and marks it as active for the current Worker
    async fn dispatch(self) -> Self::Running;
}

#[async_trait]
pub trait RunningTask: TaskState {
    type Completed: SuccessTask;
    type Failed: FailedTask;
    type Retrying: RetryingTask;

    /// The core business logic. Async allows non-blocking I/O within the task itself.
    async fn run(&mut self) -> Result<(), String>;

    /// Commits the result to the Backend and removes the task from the active queue
    async fn complete(self) -> Self::Completed;

    /// Logs the failure and updates the task status in the Broker
    async fn fail(self, reason: String) -> Self::Failed;

    /// Schedules a retry with a specific backoff delay in the Broker
    async fn retry(self, reason: String) -> Self::Retrying;
}

// --- Terminal State Definitions ---
pub trait SuccessTask: TaskState {}
pub trait FailedTask: TaskState {
    fn reason(&self) -> &str;
}
pub trait CanceledTask: TaskState {}

#[async_trait]
pub trait RetryingTask: TaskState {
    type Waiting: WaitingTask;

    /// Re-inserts the task into the Broker after the retry delay expires
    async fn reschedule(self) -> Self::Waiting;
}
