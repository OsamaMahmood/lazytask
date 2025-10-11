// Common utility functions

use std::env;
use std::path::PathBuf;

pub fn get_taskwarrior_data_dir() -> Option<PathBuf> {
    // Check environment variable first
    if let Ok(data_dir) = env::var("TASKDATA") {
        return Some(PathBuf::from(data_dir));
    }
    
    // Check home directory
    if let Some(home) = dirs::home_dir() {
        let task_dir = home.join(".task");
        if task_dir.exists() {
            return Some(task_dir);
        }
    }
    
    None
}

pub fn get_taskrc_path() -> Option<PathBuf> {
    // Check environment variable first
    if let Ok(taskrc) = env::var("TASKRC") {
        return Some(PathBuf::from(taskrc));
    }
    
    // Check home directory
    if let Some(home) = dirs::home_dir() {
        let taskrc = home.join(".taskrc");
        if taskrc.exists() {
            return Some(taskrc);
        }
    }
    
    None
}

pub fn calculate_urgency(task: &crate::data::models::Task) -> f64 {
    let mut urgency = 0.0;
    
    // Base urgency
    urgency += 1.0;
    
    // Priority urgency
    if let Some(priority) = &task.priority {
        match priority {
            crate::data::models::Priority::High => urgency += 6.0,
            crate::data::models::Priority::Medium => urgency += 3.9,
            crate::data::models::Priority::Low => urgency += 1.8,
        }
    }
    
    // Project urgency
    if task.project.is_some() {
        urgency += 1.0;
    }
    
    // Active task urgency
    if task.is_active() {
        urgency += 4.0;
    }
    
    // Tags urgency
    urgency += task.tags.len() as f64 * 1.0;
    
    // Due date urgency
    if let Some(due) = task.due {
        let now = chrono::Utc::now();
        let days_until_due = (due - now).num_days();
        
        if days_until_due < 0 {
            // Overdue
            urgency += 12.0;
        } else if days_until_due < 7 {
            // Due this week
            urgency += 5.0;
        } else if days_until_due < 30 {
            // Due this month
            urgency += 2.0;
        }
    }
    
    urgency
}
