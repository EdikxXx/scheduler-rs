use super::errors::StorageError;
use crate::types::{Task, TaskStorage};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use uuid::Uuid;

pub struct MemStorage {
    tasks: HashMap<Uuid, Task>,
}

impl MemStorage {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
}

impl TaskStorage<StorageError> for MemStorage {
    fn add(&mut self, task: Task) -> Result<Uuid, StorageError> {
        let key = task.id.clone();

        match self.tasks.entry(key) {
            Entry::Occupied(_) => Err(StorageError::DuplicateKey(task.name)),
            Entry::Vacant(entry) => {
                entry.insert(task);
                Ok(key)
            }
        }
    }

    fn remove(&mut self, id: Uuid) -> Option<Task> {
        match self.tasks.entry(id) {
            Entry::Occupied(entry) => Some(entry.remove()),
            Entry::Vacant(_) => None,
        }
    }

    fn get_by_id(&self, id: Uuid) -> Option<Task> {
        todo!()
    }

    fn get_due_tasks(&self, now: DateTime<Utc>, limit: usize) -> Vec<Uuid> {
        todo!()
    }
}
