use super::types::Broker;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

/// A thread-safe, in-memory implementation of the `Broker` trait.
///
/// Uses `tokio::sync::mpsc` for communication and `dashmap` for
/// managing multiple topics without global locking.
///
/// # Architecture
/// This broker uses a **Shared Consumer** pattern. When multiple workers
/// call `consume` for the same topic, they receive an `Arc<Mutex<Receiver>>`,
/// allowing them to compete for tasks.
pub struct InMemoryBroker<T> {
    /// Mapping of topic names to their respective Senders.
    senders: DashMap<String, mpsc::Sender<T>>,
    /// Mapping of topic names to shared, thread-safe Receivers.
    receivers: DashMap<String, Arc<Mutex<mpsc::Receiver<T>>>>,
    /// Maximum number of messages a topic can hold before blocking the publisher.
    buffer_size: usize,
}

impl<T> InMemoryBroker<T> {
    /// Creates a new `InMemoryBroker` with a specified buffer size for channels.
    pub fn new(buffer_size: usize) -> Self {
        Self {
            senders: DashMap::new(),
            receivers: DashMap::new(),
            buffer_size,
        }
    }
}

#[async_trait]
impl<T> Broker for InMemoryBroker<T>
where
    T: Send + Sync + From<Vec<u8>> + 'static,
{
    type Message = T;
    type Consumer = Arc<Mutex<mpsc::Receiver<T>>>;

    async fn publish(&self, topic: &str, message: Self::Message) -> Result<()> {
        if let Some(sender) = self.senders.get(topic) {
            sender
                .send(message)
                .await
                .map_err(|_| anyhow!("Channel closed for topic: {}", topic))?;
            Ok(())
        } else {
            Err(anyhow!("No active consumers for topic: {}", topic))
        }
    }

    async fn consume(&self, topic: &str) -> Result<Self::Consumer> {
        if let Some(rx) = self.receivers.get(topic) {
            return Ok(rx.clone());
        }

        let (tx, rx) = mpsc::channel(self.buffer_size);
        let shared_rx = Arc::new(Mutex::new(rx));

        self.senders.insert(topic.to_string(), tx);
        self.receivers.insert(topic.to_string(), shared_rx.clone());

        Ok(shared_rx)
    }

    async fn ack(&self, _message_id: &Uuid) -> Result<()> {
        Ok(())
    }
}
