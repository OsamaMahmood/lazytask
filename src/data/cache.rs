// Performance caching layer for task data

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::data::models::Task;

pub struct TaskCache {
    tasks: HashMap<String, CachedTask>,
    max_age: Duration,
}

struct CachedTask {
    task: Task,
    cached_at: Instant,
}

impl TaskCache {
    pub fn new(max_age_seconds: u64) -> Self {
        TaskCache {
            tasks: HashMap::new(),
            max_age: Duration::from_secs(max_age_seconds),
        }
    }

    pub fn get(&self, uuid: &str) -> Option<&Task> {
        if let Some(cached) = self.tasks.get(uuid) {
            if cached.cached_at.elapsed() < self.max_age {
                return Some(&cached.task);
            }
        }
        None
    }

    pub fn insert(&mut self, task: Task) {
        let uuid = task.uuid.clone();
        self.tasks.insert(uuid, CachedTask {
            task,
            cached_at: Instant::now(),
        });
    }

    pub fn remove(&mut self, uuid: &str) {
        self.tasks.remove(uuid);
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    pub fn cleanup_expired(&mut self) {
        let max_age = self.max_age;
        self.tasks.retain(|_, cached| cached.cached_at.elapsed() < max_age);
    }
}

