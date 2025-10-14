// Import/export utilities for task data

use anyhow::Result;
use serde_json;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use crate::data::models::Task;

pub enum ExportFormat {
    Json,
    Csv,
}

pub struct TaskExporter;

impl TaskExporter {
    pub fn export_to_file(tasks: &[Task], path: &Path, format: ExportFormat) -> Result<()> {
        match format {
            ExportFormat::Json => Self::export_json(tasks, path),
            ExportFormat::Csv => Self::export_csv(tasks, path),
        }
    }

    pub fn import_from_file(path: &Path, format: ExportFormat) -> Result<Vec<Task>> {
        match format {
            ExportFormat::Json => Self::import_json(path),
            ExportFormat::Csv => Self::import_csv(path),
        }
    }

    fn export_json(tasks: &[Task], path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let json = serde_json::to_string_pretty(tasks)?;
        writer.write_all(json.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    fn import_json(path: &Path) -> Result<Vec<Task>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let tasks: Vec<Task> = serde_json::from_reader(reader)?;
        Ok(tasks)
    }

    fn export_csv(tasks: &[Task], path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        // Write CSV header
        writeln!(writer, "ID,UUID,Status,Description,Project,Priority,Due,Tags")?;
        
        // Write task data
        for task in tasks {
            writeln!(
                writer,
                "{},{},{},{},{},{},{},{}",
                task.id.map(|id| id.to_string()).unwrap_or_default(),
                task.uuid,
                task.status.as_str(),
                task.description,
                task.project.as_deref().unwrap_or(""),
                task.priority.as_ref().map(|p| p.as_str()).unwrap_or(""),
                task.due.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
                task.tags.join(";")
            )?;
        }
        
        writer.flush()?;
        Ok(())
    }

    fn import_csv(_path: &Path) -> Result<Vec<Task>> {
        // TODO: Implement CSV import
        todo!("CSV import not yet implemented")
    }
}

