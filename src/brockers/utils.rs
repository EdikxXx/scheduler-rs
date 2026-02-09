use super::types::Broker;
use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct InMemoryBroker<T> {
    topics: DashMap<String, mpsc::Sender<T>>,
    buffer_size: usize,
}

impl<T> InMemoryBroker<T> {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            topics: DashMap::new(),
            buffer_size,
        }
    }

    /// Method for a worker to “subscribe” to a topic and receive a Receiver
    pub fn subscribe(&self, topic: &str) -> mpsc::Receiver<T> {
        let (tx, rx) = mpsc::channel(self.buffer_size);
        self.topics.insert(topic.to_string(), tx);
        rx
    }
}

#[async_trait]
impl<T> Broker for InMemoryBroker<T>
where
    T: Send + Sync + From<Vec<u8>>,
{
    type Message = T;
    type Consumer = Arc<mpsc::Receiver<T>>;

    async fn publish(&self, topic: &str, message: Self::Message) -> Result<()> {
        if let Some(sender) = self.topics.get(topic) {
            sender
                .send(message)
                .await
                .map_err(|_| anyhow::anyhow!("Channel closed for topic: {}", topic))?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("No subscribers for topic: {}", topic))
        }
    }
    async fn consume(&self, topic: &str) -> Result<Self::Consumer> {
        todo!()
    }

    async fn ack(&self, _message_id: &Uuid) -> Result<()> {
        todo!()
    }
}
