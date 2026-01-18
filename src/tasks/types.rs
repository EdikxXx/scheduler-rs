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
    id: Uuid,
    name: String,
    last_run: Option<DateTime<Utc>>,
    kind: TaskKind,
}

impl Task {
    pub fn new(name: &str, kind: TaskKind) -> Self {
        Self {
            id: Uuid::nil(),
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

pub trait TaskStorage {
    fn add(&mut self, task: Task) -> Uuid;
    fn remove(&mut self, id: Uuid) -> bool;

    fn get_by_id(&self, id: Uuid) -> Option<Task>;
    fn get_due_tasks(&self, now: DateTime<Utc>, limit: usize) -> Vec<Uuid>;
}
