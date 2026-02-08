use std::sync::Arc;

use super::types::*;
use crate::brockers::types::Broker;
use anyhow::Result;
use async_trait::async_trait;
use serde_json;
use std::marker::PhantomData;
use uuid::Uuid;

macro_rules! change_state {
    ($task:expr, $new_state:ty) => {
        Task {
            id: $task.id,
            name: $task.name,
            pyload: $task.pyload,
            state: std::marker::PhantomData::<$new_state>,
        }
    };
}

pub struct Task<State, Pyload> {
    id: Uuid,
    name: String,
    pyload: Pyload,
    state: PhantomData<State>,
}

impl<P> Task<Pending, P>
where
    P: TaskData,
{
    pub fn new(id: Uuid, name: &str, pyload: P) -> Self {
        Self {
            id,
            name: name.to_string(),
            pyload,
            state: PhantomData,
        }
    }
}

impl<S, P> TaskState for Task<S, P>
where
    S: Send + Sync,
    P: TaskData,
{
    type Payload = P;
    type State = S;

    fn id(&self) -> Uuid {
        self.id
    }

    fn payload(&self) -> &Self::Payload {
        &self.pyload
    }

    fn task_name(&self) -> &str {
        &self.name
    }
}

#[async_trait]
impl<P> PendingTask for Task<Pending, P>
where
    P: TaskData,
{
    type WaitingOutput = Task<Waiting, P>;
    type CanceledOutput = Task<Canceled, P>;

    async fn submit<B: Broker>(self, broker: Arc<B>) -> Result<Self::WaitingOutput> {
        let queue = P::queue_name();
        let payload_bytes = serde_json::to_vec(&self.pyload)?;
        broker.publish(queue, payload_bytes.into()).await?;
        Ok(change_state!(self, Waiting))
    }

    async fn cancel(self) -> Self::CanceledOutput {
        change_state!(self, Canceled)
    }
}

#[async_trait]
impl<P> WaitingTask for Task<Waiting, P>
where
    P: TaskData,
{
    type RunningOutput = Task<Running, P>;

    async fn dispatch(self) -> Self::RunningOutput {
        //TODO dispatch logic
        change_state!(self, Running)
    }
}

#[async_trait]
impl<P> RunningTask for Task<Running, P>
where
    P: TaskData,
{
    type CompletedOutput = Task<Success, P>;
    type FailedOutput = Task<Failed, P>;
    type RetryingOutput = Task<Retrying, P>;

    async fn run(&mut self) -> Result<()> {
        self.pyload.execute().await
    }

    async fn complete(self) -> Self::CompletedOutput {
        todo!()
    }

    async fn fail(self, reason: String) -> Self::FailedOutput {
        todo!()
    }

    async fn retry(self, reason: String) -> Self::RetryingOutput {
        todo!()
    }
}

impl<P> SuccessTask for Task<Success, P> where P: TaskData {}

impl<P> CanceledTask for Task<Canceled, P> where P: TaskData {}

impl<P> FailedTask for Task<Failed, P>
where
    P: TaskData,
{
    fn reason(&self) -> &str {
        todo!()
    }
}

#[async_trait]
impl<P> RetryingTask for Task<Retrying, P>
where
    P: TaskData,
{
    type WaitingOutput = Task<Waiting, P>;
    async fn reschedule(self) -> Self::WaitingOutput {
        todo!()
    }
}
