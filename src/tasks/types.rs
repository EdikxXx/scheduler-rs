use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use time::OffsetDateTime as DateTime;
use uuid::Uuid;

pub type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type Job = Box<dyn Fn() -> BoxedFuture + Sync + Send>;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum TaskKind {
    Interval(Duration),
    Once(DateTime),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub last_run: Option<DateTime>,
    pub kind: TaskKind,
}

impl Task {
    pub fn new(name: &str, kind: TaskKind) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: name.to_string(),
            last_run: None,
            kind,
        }
    }

    pub fn next_run(&self) -> DateTime {
        match self.kind {
            TaskKind::Interval(dur) => match self.last_run {
                Some(last) => last + dur,
                None => DateTime::now_utc(),
            },
            TaskKind::Once(run_time) => run_time,
        }
    }
}

pub trait TaskStorage<E> {
    fn add(&self, task: Task) -> Result<Uuid, E>;
    fn remove(&self, id: Uuid) -> Result<Option<Task>, E>;
    fn get_by_id(&self, id: Uuid) -> Result<Option<Task>, E>;
}

#[async_trait]
pub trait AsyncTaskStorage<E> {
    async fn add(&mut self, task: Task) -> Result<Uuid, E>;
    async fn remove(&mut self, id: &Uuid) -> bool;

    async fn get_by_id(&self, id: Uuid) -> Option<Task>;
    async fn get_due_tasks(&self, now: DateTime, limit: usize) -> Vec<Uuid>;
}
