// Date/text formatting utilities

use chrono::{DateTime, Local, Utc};

pub fn format_date(date: &DateTime<Utc>) -> String {
    let local_date = date.with_timezone(&Local);
    local_date.format("%Y-%m-%d").to_string()
}

pub fn format_datetime(datetime: &DateTime<Utc>) -> String {
    let local_datetime = datetime.with_timezone(&Local);
    local_datetime.format("%Y-%m-%d %H:%M").to_string()
}

pub fn format_relative_date(date: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*date);
    
    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m ago", duration.num_minutes())
    } else {
        "Just now".to_string()
    }
}

pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length.saturating_sub(3)])
    }
}
