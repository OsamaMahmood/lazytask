// Demo program showing current LazyTask functionality

use anyhow::Result;
use lazytask::taskwarrior::TaskwarriorIntegration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎉 LazyTask v0.1.0 - Demo");
    println!("========================");
    println!();

    let taskwarrior = TaskwarriorIntegration::new(None, None)?;

    // Show current task list
    println!("📋 Current Tasks:");
    let tasks = taskwarrior.list_tasks(Some("+PENDING")).await?;
    if tasks.is_empty() {
        println!("   No pending tasks found.");
    } else {
        for task in &tasks {
            let priority = task.priority.as_ref()
                .map(|p| format!("[{}]", p.as_char()))
                .unwrap_or("[ ]".to_string());
            let project = task.project.as_deref().unwrap_or("(no project)");
            let due = task.due
                .map(|d| format!("due:{}", d.format("%Y-%m-%d")))
                .unwrap_or_default();
            
            println!("   {} {} {} {} - {}", 
                task.id.map(|id| format!("{:2}", id)).unwrap_or("  ".to_string()),
                priority,
                format!("{:12}", project),
                format!("{:12}", due),
                task.description
            );
        }
    }

    println!();
    println!("🚀 LazyTask Features Implemented:");
    println!("  ✅ Task Loading - Display real Taskwarrior tasks");
    println!("  ✅ Task Navigation - Arrow key navigation with selection");
    println!("  ✅ Task Creation - Modal form for adding new tasks");
    println!("  ✅ Task Completion - Mark tasks as done with 'd' key");
    println!("  ✅ Task Deletion - Delete tasks with confirmation");
    println!("  ✅ Filtering Support - Load tasks with filters");
    println!("  ✅ Theme System - Beautiful Catppuccin color scheme");
    println!("  ✅ Configuration - TOML-based customizable settings");
    println!("  ✅ Help System - Context-sensitive help (F1)");
    println!("  ✅ Auto-refresh - UI updates after operations");
    
    println!();
    println!("⌨️  Current Keyboard Shortcuts:");
    println!("  q         - Quit application");
    println!("  F1        - Show help");  
    println!("  F5        - Refresh tasks");
    println!("  ↑/↓       - Navigate tasks");
    println!("  a         - Add new task");
    println!("  d         - Mark task done");
    println!("  Delete    - Delete task");
    println!("  Esc       - Go back/cancel");
    println!("  Enter     - Select/confirm");

    println!();
    println!("🧪 To test the TUI interface:");
    println!("   cargo run");
    println!();
    println!("📚 Next Phase Ready:");
    println!("  🔄 Interactive filter engine with real-time preview");
    println!("  🔄 Split-panel interface (list + detail views)");
    println!("  🔄 Reports dashboard with calendar integration");
    println!("  🔄 Advanced keybinding customization");
    
    Ok(())
}

