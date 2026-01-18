use crate::types::{Task, TaskStorage};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct MemStorage {
    tasks: Vec<Task>,
}

impl MemStorage {
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }
}

impl TaskStorage for MemStorage {
    fn add(&mut self, task: Task) -> Uuid {
        todo!()
    }

    fn remove(&mut self, id: Uuid) -> bool {
        todo!()
    }

    fn get_by_id(&self, id: Uuid) -> Option<Task> {
        todo!()
    }
    fn get_due_tasks(&self, now: DateTime<Utc>, limit: usize) -> Vec<Uuid> {
        todo!()
    }
}
