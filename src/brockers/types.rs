use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait Broker: Send + Sync {
    /// The message type that travels through the broker
    type Message: Send + Sync + From<Vec<u8>>;
    type Consumer: Send + Sync;

    /// Sends a message into the pipeline (Exchange/Queue)
    async fn publish(&self, topic: &str, message: Self::Message) -> Result<()>;

    /// Subscribes to a stream of messages from a specific topic/queue
    /// In Rust, this would typically return a Stream
    async fn consume(&self, topic: &str) -> Result<Self::Consumer>;

    /// Acknowledges that a message has been processed
    /// (Crucial for reliability - the "Ack" signal)
    async fn ack(&self, message_id: &Uuid) -> Result<()>;
}
