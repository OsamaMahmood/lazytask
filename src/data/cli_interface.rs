// Taskwarrior CLI wrapper for executing task commands

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::data::models::Task;

pub struct TaskwarriorCLI {
    taskrc_path: Option<PathBuf>,
}

impl TaskwarriorCLI {
    pub fn new(taskrc_path: Option<PathBuf>) -> Self {
        TaskwarriorCLI { taskrc_path }
    }

    pub async fn list_tasks(&self, filter: Option<&str>) -> Result<Vec<Task>> {
        let mut args = vec!["export"];
        if let Some(f) = filter {
            args.insert(0, f);
        }

        let output = self.execute_command(&args)?;
        let tasks: Vec<serde_json::Value> = serde_json::from_str(&output)
            .with_context(|| "Failed to parse task export JSON")?;

        let mut result = Vec::new();
        for task_json in tasks {
            if let Ok(task) = Task::from_json(&task_json) {
                result.push(task);
            }
        }

        Ok(result)
    }

    pub async fn add_task(&self, description: &str, attributes: &[(&str, &str)]) -> Result<u32> {
        let mut args = vec!["add".to_string(), description.to_string()];
        
        for (key, value) in attributes {
            args.push(format!("{}:{}", key, value));
        }

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = self.execute_command(&args_refs)?;
        
        // Parse the task ID from output like "Created task 42."
        let id_str = output
            .split_whitespace()
            .find(|word| word.chars().all(|c| c.is_ascii_digit()))
            .ok_or_else(|| anyhow::anyhow!("Could not parse task ID from output: {}", output))?;
        
        id_str.parse().with_context(|| "Failed to parse task ID")
    }

    fn execute_command(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("task");
        
        if let Some(taskrc) = &self.taskrc_path {
            cmd.arg("rc:").arg(taskrc);
        }
        
        cmd.args(args);
        
        let output = cmd.output()
            .with_context(|| format!("Failed to execute task command: {:?}", args))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Task command failed: {}", error));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
