use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use uuid::Uuid;

pub type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type Job = Box<dyn Fn() -> BoxedFuture + Sync + Send>;

#[derive(Serialize, Deserialize, Debug)]
pub enum TaskKind {
    Interval(Duration),
    Once(DateTime<Utc>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub name: String,
    pub last_run: Option<DateTime<Utc>>,
    pub kind: TaskKind,
}

impl Task {
    pub fn new(name: &str, kind: TaskKind) -> Self {
        Self {
            name: name.to_string(),
            last_run: None,
            kind,
        }
    }

    pub fn next_run(&mut self) -> DateTime<Utc> {
        match self.kind {
            TaskKind::Interval(dur) => match self.last_run {
                Some(last) => last + dur,
                None => Utc::now(),
            },
            TaskKind::Once(run_time) => run_time,
        }
    }
}

pub trait TaskStorage<T, E> {
    fn add(&mut self, task: Task) -> Result<T, E>;
    fn remove(&mut self, id: T) -> bool;

    fn get_by_id(&self, id: T) -> Option<Task>;
    fn get_due_tasks(&self, now: DateTime<Utc>, limit: usize) -> Vec<T>;
}

#[async_trait]
pub trait AsyncTaskStorage<T, E> {
    async fn add(&mut self, task: Task) -> Result<T, E>;
    async fn remove(&mut self, id: T) -> bool;

    async fn get_by_id(&self, id: T) -> Option<Task>;
    async fn get_due_tasks(&self, now: DateTime<Utc>, limit: usize) -> Vec<T>;
}
