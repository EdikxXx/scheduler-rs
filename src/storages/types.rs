use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send + Sync {
    // The concrete task type this storage handles
    type Item;

    /// Pushes a task into the storage (e.g., LPUSH in Redis)
    async fn push(&self, task: Self::Item) -> Result<()>;

    /// Searches for a task based on a predicate.
    /// In EDA, this is often used for Dead Letter Queues or task inspection.
    async fn search<F>(&self, mut filter: F) -> Result<Option<Self::Item>>
    where
        F: FnMut(&Self::Item) -> bool + Send;
}
