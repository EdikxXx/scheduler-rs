use super::errors::StorageError;
use crate::types::{Task, TaskStorage};
use dashmap::{DashMap, mapref::entry::Entry};
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

type TaskRef = Arc<RwLock<Task>>;

pub struct MemStorage {
    tasks: DashMap<Uuid, TaskRef>,
}

impl MemStorage {
    fn new() -> Self {
        Self {
            tasks: DashMap::new(),
        }
    }
}

impl TaskStorage<StorageError> for MemStorage {
    fn add(&self, task: Task) -> Result<Uuid, StorageError> {
        let key = task.id;

        match self.tasks.entry(key) {
            Entry::Occupied(_) => Err(StorageError::DuplicateKey(task.name)),
            Entry::Vacant(entry) => {
                entry.insert(Arc::new(RwLock::new(task)));
                Ok(key)
            }
        }
    }

    fn remove(&self, id: Uuid) -> Result<Option<Task>, StorageError> {
        let maybe_task_arc = self.tasks.remove(&id).map(|(_uuid, arc)| arc);

        let task_arc = match maybe_task_arc {
            Some(arc) => arc,
            None => return Ok(None),
        };

        let task = match Arc::try_unwrap(task_arc) {
            Ok(rwlock) => rwlock.into_inner(),

            Err(arc_ref) => {
                let lock = arc_ref.read();
                (*lock).clone()
            }
        };

        Ok(Some(task))
    }

    fn get_by_id(&self, id: Uuid) -> Result<Option<Task>, StorageError> {
        todo!()
    }
}
