use super::errors::StorageError;
use crate::types::{Task, TaskStorage};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub struct MemStorage {
    tasks: HashMap<String, Task>,
}

impl MemStorage {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
}

impl TaskStorage<String, StorageError> for MemStorage {
    fn add(&mut self, task: Task) -> Result<String, StorageError> {
        let key = task.name.clone();

        match self.tasks.entry(key) {
            Entry::Occupied(entry) => Err(StorageError::DuplicateKey(task.name)),
            Entry::Vacant(entry) => {
                let return_value = entry.key().clone();
                entry.insert(task);
                Ok(return_value)
            }
        }
    }

    fn remove(&mut self, id: String) -> bool {
        todo!()
    }

    fn get_by_id(&self, id: String) -> Option<Task> {
        todo!()
    }
    fn get_due_tasks(&self, now: DateTime<Utc>, limit: usize) -> Vec<String> {
        todo!()
    }
}
