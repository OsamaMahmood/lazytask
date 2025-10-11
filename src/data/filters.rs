// Query and filter engine for tasks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::models::{Task, TaskStatus, Priority};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub project: Option<String>,
    pub priority: Option<Priority>,
    pub due_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub description_contains: Option<String>,
    pub is_active: Option<bool>,
    pub is_overdue: Option<bool>,
    pub is_blocked: Option<bool>,
}

impl Default for TaskFilter {
    fn default() -> Self {
        TaskFilter {
            status: Some(TaskStatus::Pending),
            project: None,
            priority: None,
            due_before: None,
            due_after: None,
            tags: Vec::new(),
            description_contains: None,
            is_active: None,
            is_overdue: None,
            is_blocked: None,
        }
    }
}

impl TaskFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn matches(&self, task: &Task) -> bool {
        // Status filter
        if let Some(status) = &self.status {
            if &task.status != status {
                return false;
            }
        }

        // Project filter
        if let Some(project) = &self.project {
            match &task.project {
                Some(task_project) => {
                    if !task_project.contains(project) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Priority filter
        if let Some(priority) = &self.priority {
            match &task.priority {
                Some(task_priority) => {
                    if task_priority != priority {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Due date filters
        if let Some(due_before) = self.due_before {
            match task.due {
                Some(task_due) => {
                    if task_due >= due_before {
                        return false;
                    }
                }
                None => return false,
            }
        }

        if let Some(due_after) = self.due_after {
            match task.due {
                Some(task_due) => {
                    if task_due <= due_after {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Tags filter
        if !self.tags.is_empty() {
            for required_tag in &self.tags {
                if !task.tags.contains(required_tag) {
                    return false;
                }
            }
        }

        // Description contains filter
        if let Some(text) = &self.description_contains {
            if !task.description.to_lowercase().contains(&text.to_lowercase()) {
                return false;
            }
        }

        // Special filters
        if let Some(active) = self.is_active {
            if task.is_active() != active {
                return false;
            }
        }

        if let Some(overdue) = self.is_overdue {
            if task.is_overdue() != overdue {
                return false;
            }
        }

        if let Some(blocked) = self.is_blocked {
            if task.is_blocked() != blocked {
                return false;
            }
        }

        true
    }

    pub fn apply(&self, tasks: &[Task]) -> Vec<Task> {
        tasks.iter()
            .filter(|task| self.matches(task))
            .cloned()
            .collect()
    }
}
