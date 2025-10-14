// Input validation utilities

use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, Utc};

pub fn validate_task_description(description: &str) -> Result<()> {
    if description.trim().is_empty() {
        return Err(anyhow!("Task description cannot be empty"));
    }
    
    if description.len() > 1000 {
        return Err(anyhow!("Task description is too long (max 1000 characters)"));
    }
    
    Ok(())
}

pub fn validate_project_name(project: &str) -> Result<()> {
    if project.is_empty() {
        return Err(anyhow!("Project name cannot be empty"));
    }
    
    if project.contains(' ') {
        return Err(anyhow!("Project name cannot contain spaces"));
    }
    
    if !project.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-') {
        return Err(anyhow!("Project name can only contain alphanumeric characters, dots, underscores, and hyphens"));
    }
    
    Ok(())
}

pub fn validate_tag_name(tag: &str) -> Result<()> {
    if tag.is_empty() {
        return Err(anyhow!("Tag name cannot be empty"));
    }
    
    if tag.contains(' ') {
        return Err(anyhow!("Tag name cannot contain spaces"));
    }
    
    if !tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(anyhow!("Tag name can only contain alphanumeric characters, underscores, and hyphens"));
    }
    
    Ok(())
}

pub fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    // Try parsing different date formats
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }
    
    if let Ok(datetime) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(datetime.with_timezone(&Utc));
    }
    
    Err(anyhow!("Invalid date format. Use YYYY-MM-DD or RFC3339 format"))
}

