use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Scheduler: Send + Sync {
    /// The backing storage engine (e.g., Redis, Postgres, or In-memory)
    type Storage: Send + Sync;

    /// The main entry point. Orchestrates the infinite event loop.
    /// It checks the schedule and triggers events when the time comes.
    async fn run(&self) -> Result<()>;

    /// Dispatches a specific event/task to the Broker.
    /// This is called internally by `run` when a timer expires.
    async fn send_event(&self) -> Result<()>;

    /// Provides access to the underlying storage for task inspection or manual triggers.
    async fn get_storage(&self) -> &Self::Storage;
}
