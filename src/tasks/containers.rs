use crate::types::Job;
use std::collections::HashMap;

/// Container for storage map task name and an execute code
#[derive(Default)] // Упрощает new()
pub struct HandlerRegistry {
    pub map: HashMap<String, Job>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert_task(&mut self, name: &str, task: Job) {
        self.map.insert(name.to_string(), task);
    }

    pub async fn run(&self, name: &str) {
        if let Some(job) = self.map.get(name) {
            // Вызываем функцию, получаем Future и ждем её (await)
            job().await;
        } else {
            println!("Task not found");
        }
    }
}
