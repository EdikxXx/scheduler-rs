use std::sync::Arc;

use crate::brockers::types::Broker;
use anyhow::Result;
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

// --- Execute Interface ---
/// The provider for execution
#[async_trait]
pub trait TaskData: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// Queue name where task is be
    fn queue_name() -> &'static str;

    /// Task execute logic
    async fn execute(&self) -> Result<()>;
}

// --- Base Interface ---
/// The "Blueprint" of your Task in a Distributed System
pub trait TaskState: Send + Sync {
    /// Unique Task Data (Must be serializable to cross the Broker boundary)
    type Payload: Send + Sync + Serialize + for<'de> Deserialize<'de>;

    /// Current State Marker (Pending, Running, etc.)
    type State: Send + Sync;

    fn id(&self) -> Uuid;
    fn payload(&self) -> &Self::Payload;
    fn task_name(&self) -> &str;
}

// --- Lifecycle Transitions (Async) ---

#[async_trait]
pub trait PendingTask: TaskState {
    type WaitingOutput: WaitingTask;
    type CanceledOutput: CanceledTask;

    /// Sends the task payload to the Message Broker (Network I/O)
    async fn submit<B: Broker>(self, broker: Arc<B>) -> Result<Self::WaitingOutput>;

    /// Flags the task as canceled in the local or remote registry
    async fn cancel(self) -> Self::CanceledOutput;
}

#[async_trait]
pub trait WaitingTask: TaskState {
    type RunningOutput: RunningTask;

    /// Fetches the task from the queue and marks it as active for the current Worker
    async fn dispatch(self) -> Self::RunningOutput;
}

#[async_trait]
pub trait RunningTask: TaskState {
    type CompletedOutput: SuccessTask;
    type FailedOutput: FailedTask;
    type RetryingOutput: RetryingTask;

    /// The core business logic. Async allows non-blocking I/O within the task itself.
    async fn run(&mut self) -> Result<()>;

    /// Commits the result to the Backend and removes the task from the active queue
    async fn complete(self) -> Self::CompletedOutput;

    /// Logs the failure and updates the task status in the Broker
    async fn fail(self, reason: String) -> Self::FailedOutput;

    /// Schedules a retry with a specific backoff delay in the Broker
    async fn retry(self, reason: String) -> Self::RetryingOutput;
}

// --- Terminal State Definitions ---
pub trait SuccessTask: TaskState {}
pub trait FailedTask: TaskState {
    fn reason(&self) -> &str;
}
pub trait CanceledTask: TaskState {}

#[async_trait]
pub trait RetryingTask: TaskState {
    type WaitingOutput: WaitingTask;

    /// Re-inserts the task into the Broker after the retry delay expires
    async fn reschedule(self) -> Self::WaitingOutput;
}
