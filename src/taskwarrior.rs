use anyhow::{Context, Result};
use rusqlite::Connection;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

use crate::data::models::Task;

pub struct TaskwarriorIntegration {
    cli: TaskwarriorCLI,
    db: Option<TaskChampionDB>,
}

pub struct TaskwarriorCLI {
    taskrc_path: Option<PathBuf>,
}

pub struct TaskChampionDB {
    conn: Connection,
}

impl TaskwarriorIntegration {
    pub fn new(taskrc_path: Option<PathBuf>, data_location: Option<PathBuf>) -> Result<Self> {
        let cli = TaskwarriorCLI::new(taskrc_path.clone());
        
        let db = if let Some(data_path) = data_location {
            let db_path = data_path.join("taskchampion.sqlite3");
            if db_path.exists() {
                Some(TaskChampionDB::new(db_path)?)
            } else {
                None
            }
        } else {
            // Try to find the default data location
            if let Ok(data_path) = Self::get_data_location(&cli) {
                let db_path = PathBuf::from(data_path).join("taskchampion.sqlite3");
                if db_path.exists() {
                    Some(TaskChampionDB::new(db_path)?)
                } else {
                    None
                }
            } else {
                None
            }
        };

        Ok(TaskwarriorIntegration { cli, db })
    }

    pub async fn list_tasks(&self, filter: Option<&str>) -> Result<Vec<Task>> {
        // For now, always use CLI since DB implementation is not complete
        // TODO: Implement direct database access for better performance
        self.cli.list_tasks(filter).await
    }

    pub async fn get_task(&self, id: u32) -> Result<Option<Task>> {
        // For now, always use CLI since DB implementation is not complete
        self.cli.get_task(id).await
    }

    pub async fn add_task(&self, description: &str, attributes: &[(&str, &str)]) -> Result<u32> {
        self.cli.add_task(description, attributes).await
    }

    pub async fn modify_task(&self, id: u32, attributes: &[(&str, &str)]) -> Result<()> {
        self.cli.modify_task(id, attributes).await
    }

    pub async fn done_task(&self, id: u32) -> Result<()> {
        self.cli.done_task(id).await
    }

    pub async fn delete_task(&self, id: u32) -> Result<()> {
        self.cli.delete_task(id).await
    }

    fn get_data_location(cli: &TaskwarriorCLI) -> Result<String> {
        cli.execute_command(&["_get", "rc.data.location"])
    }
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
        let tasks: Vec<Value> = serde_json::from_str(&output)
            .with_context(|| "Failed to parse task export JSON")?;

        let mut result = Vec::new();
        for task_json in tasks {
            if let Ok(task) = Task::from_json(&task_json) {
                result.push(task);
            }
        }

        Ok(result)
    }

    pub async fn get_task(&self, id: u32) -> Result<Option<Task>> {
        let filter = &format!("{}", id);
        let tasks = self.list_tasks(Some(filter)).await?;
        Ok(tasks.into_iter().next())
    }

    pub async fn add_task(&self, description: &str, attributes: &[(&str, &str)]) -> Result<u32> {
        let mut args = vec!["add".to_string(), description.to_string()];
        
        for (key, value) in attributes {
            if value.is_empty() {
                // For tags and other attributes without values (like +tag)
                args.push(key.to_string());
            } else {
                // For attributes with values (like project:name, priority:H)
                args.push(format!("{}:{}", key, value));
            }
        }

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = self.execute_command(&args_refs)?;
        
        // Parse the task ID from output like "Created task 42."
        let id_str = output
            .split_whitespace()
            .find(|word| word.ends_with('.') && word[..word.len()-1].chars().all(|c| c.is_ascii_digit()))
            .map(|word| &word[..word.len()-1])  // Remove the trailing dot
            .or_else(|| output.split_whitespace().find(|word| word.chars().all(|c| c.is_ascii_digit())))
            .ok_or_else(|| anyhow::anyhow!("Could not parse task ID from output: {}", output))?;
        
        id_str.parse().with_context(|| "Failed to parse task ID")
    }

    pub async fn modify_task(&self, id: u32, attributes: &[(&str, &str)]) -> Result<()> {
        let mut args = vec![id.to_string(), "modify".to_string()];
        
        for (key, value) in attributes {
            if value.is_empty() {
                // Special case: for clearing attributes like "tags:", "project:", etc.
                if *key == "tags" || *key == "project" || *key == "priority" || *key == "due" {
                    args.push(format!("{}:", key));
                } else {
                    // For tags without values (like +tag)
                    args.push(key.to_string());
                }
            } else {
                // For attributes with values (like project:name, priority:H)
                args.push(format!("{}:{}", key, value));
            }
        }

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        
        self.execute_command(&args_refs)?;
        Ok(())
    }

    pub async fn done_task(&self, id: u32) -> Result<()> {
        let id_str = id.to_string();
        self.execute_command(&[&id_str, "done"])?;
        Ok(())
    }

    pub async fn delete_task(&self, id: u32) -> Result<()> {
        let id_str = id.to_string();
        // Use rc.confirmation=no to avoid interactive confirmation prompt
        self.execute_command(&[&id_str, "delete", "rc.confirmation=no"])?;
        Ok(())
    }

    fn execute_command(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("task");
        
        if let Some(taskrc) = &self.taskrc_path {
            cmd.arg(format!("rc:{}", taskrc.display()));
        }
        
        cmd.args(args);
        
        let output = cmd.output()
            .with_context(|| format!("Failed to execute task command: {:?}", args))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            // Provide detailed error information
            let error_msg = if stderr.is_empty() {
                if stdout.is_empty() {
                    format!("Task command failed with no output. Command: task {}", args.join(" "))
                } else {
                    format!("Task command failed. Output: {}", stdout)
                }
            } else {
                format!("Task command failed: {}", stderr)
            };
            return Err(anyhow::anyhow!("{}", error_msg));
        }

        Ok(stdout)
    }
}

impl TaskChampionDB {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| "Failed to open TaskChampion database")?;
        
        Ok(TaskChampionDB { conn })
    }

    pub async fn list_tasks(&self, _filter: Option<&str>) -> Result<Vec<Task>> {
        // Placeholder implementation - would need to understand TaskChampion schema
        // For now, fall back to CLI implementation
        todo!("Direct TaskChampion DB access not yet implemented")
    }

    pub async fn get_task(&self, _id: u32) -> Result<Option<Task>> {
        // Placeholder implementation
        todo!("Direct TaskChampion DB access not yet implemented")
    }
}
