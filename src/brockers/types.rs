use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// The `Broker` trait defines the interface for a message transport layer.
///
/// It acts as a bridge between the task scheduler and the workers,
/// allowing tasks to be distributed across different queues (topics).
#[async_trait]
pub trait Broker: Send + Sync {
    /// The concrete type of the message being sent.
    /// Must be convertible from a byte vector for serialization support.
    type Message: Send + Sync + From<Vec<u8>>;

    /// The consumer type returned by the `consume` method.
    /// Typically represents a stream or a shared channel receiver.
    type Consumer: Send + Sync;

    /// Publishes a message to a specific topic or queue.
    ///
    /// # Errors
    /// Returns an error if the underlying transport is unavailable or
    /// if the message cannot be delivered.
    async fn publish(&self, topic: &str, message: Self::Message) -> Result<()>;

    /// Subscribes to a topic and returns a `Consumer` to read messages.
    ///
    /// This method allows workers to pull tasks from the broker.
    /// If multiple workers consume from the same topic, the broker should
    /// ideally distribute tasks among them (e.g., Round-Robin).
    async fn consume(&self, topic: &str) -> Result<Self::Consumer>;

    /// Acknowledges the successful processing of a message.
    ///
    /// This is vital for reliability. If a message is not acknowledged,
    /// a broker may choose to re-deliver it.
    async fn ack(&self, message_id: &Uuid) -> Result<()>;
}
