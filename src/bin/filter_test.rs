// Test program to verify status filtering functionality

use anyhow::Result;
use lazytask::data::filters::TaskFilter;
use lazytask::taskwarrior::TaskwarriorIntegration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 LazyTask Filter System Test");
    println!("===============================");
    println!();

    let taskwarrior = TaskwarriorIntegration::new(None, None)?;

    // Load all tasks
    let all_tasks = taskwarrior.list_tasks(None).await?;
    println!("📋 Loaded {} total tasks", all_tasks.len());
    println!();

    // Test different filter types
    println!("🧪 Testing Status Filters:");
    println!();

    // Test pending filter
    let pending_filter = TaskFilter {
        status: Some(lazytask::data::models::TaskStatus::Pending),
        ..TaskFilter::default()
    };
    let pending_tasks = pending_filter.apply(&all_tasks);
    println!("✅ Pending filter: {} tasks", pending_tasks.len());

    // Test active filter (pending + started)
    let active_filter = TaskFilter {
        status: Some(lazytask::data::models::TaskStatus::Pending),
        is_active: Some(true),
        ..TaskFilter::default()
    };
    let active_tasks = active_filter.apply(&all_tasks);
    println!("✅ Active filter: {} tasks (pending + started)", active_tasks.len());

    // Test overdue filter (pending + past due date)
    let overdue_filter = TaskFilter {
        status: Some(lazytask::data::models::TaskStatus::Pending),
        is_overdue: Some(true),
        ..TaskFilter::default()
    };
    let overdue_tasks = overdue_filter.apply(&all_tasks);
    println!("✅ Overdue filter: {} tasks (pending + past due)", overdue_tasks.len());

    // Test completed filter
    let completed_filter = TaskFilter {
        status: Some(lazytask::data::models::TaskStatus::Completed),
        ..TaskFilter::default()
    };
    let completed_tasks = completed_filter.apply(&all_tasks);
    println!("✅ Completed filter: {} tasks", completed_tasks.len());

    // Test deleted filter
    let deleted_filter = TaskFilter {
        status: Some(lazytask::data::models::TaskStatus::Deleted),
        ..TaskFilter::default()
    };
    let deleted_tasks = deleted_filter.apply(&all_tasks);
    println!("✅ Deleted filter: {} tasks", deleted_tasks.len());

    println!();
    println!("🎛️ Status Filter Controls in LazyTask:");
    println!("  • Press '/' to open filter bar");
    println!("  • Navigate to Status field with ↑↓");
    println!("  • Press 'Space' to cycle: All → Pending → Active → Overdue → Completed → Waiting → Deleted");
    println!("  • Or type: 'p'ending, 'a'ctive/'a'll, 'o'verdue, 'c'ompleted, 'w'aiting, 'd'eleted");
    println!("  • Press 'Enter' to apply filter");
    println!("  • Press 'Esc' to close filter bar");
    println!();

    // Show sample of each type for verification
    if !active_tasks.is_empty() {
        println!("📌 Sample Active Tasks:");
        for task in active_tasks.iter().take(3) {
            println!("  • [{}] {} - {} {}", 
                task.id.unwrap_or(0),
                task.project.as_deref().unwrap_or("(none)"),
                task.description,
                if task.start.is_some() { "(STARTED)" } else { "" }
            );
        }
        println!();
    }

    if !overdue_tasks.is_empty() {
        println!("⚠️ Sample Overdue Tasks:");
        for task in overdue_tasks.iter().take(3) {
            let due_str = task.due.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or("No due date".to_string());
            println!("  • [{}] {} - {} (due: {})", 
                task.id.unwrap_or(0),
                task.project.as_deref().unwrap_or("(none)"),
                task.description,
                due_str
            );
        }
        println!();
    }

    println!("✅ Status filter system ready!");
    println!("   Press '/' in LazyTask to test the interactive filtering.");

    Ok(())
}
