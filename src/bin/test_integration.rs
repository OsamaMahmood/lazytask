// Test program to verify Taskwarrior integration

use anyhow::Result;
use lazytask::taskwarrior::TaskwarriorIntegration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing LazyTask - Taskwarrior Integration");
    println!("=========================================");

    // Initialize Taskwarrior integration
    let taskwarrior = TaskwarriorIntegration::new(None, None)?;
    
    // Test loading tasks
    println!("\n1. Loading tasks from Taskwarrior...");
    let tasks = taskwarrior.list_tasks(None).await?;
    println!("✅ Successfully loaded {} tasks", tasks.len());

    // Display tasks
    if !tasks.is_empty() {
        println!("\n2. Task list:");
        for task in &tasks[..std::cmp::min(tasks.len(), 5)] {
            let priority = task.priority.as_ref()
                .map(|p| p.as_char().to_string())
                .unwrap_or(" ".to_string());
            let project = task.project.as_deref().unwrap_or("(none)");
            println!("   [{}] {} [{}] {} - {}", 
                task.id.unwrap_or(0),
                priority,
                project,
                task.status.as_str(),
                task.description
            );
        }
        if tasks.len() > 5 {
            println!("   ... and {} more tasks", tasks.len() - 5);
        }
    }

    // Test filtering
    println!("\n3. Testing filtered tasks (project:lazytask)...");
    let filtered_tasks = taskwarrior.list_tasks(Some("project:lazytask")).await?;
    println!("✅ Found {} lazytask project tasks", filtered_tasks.len());

    for task in &filtered_tasks {
        let priority = task.priority.as_ref()
            .map(|p| p.as_char().to_string())
            .unwrap_or(" ".to_string());
        println!("   [{}] {} {} - {}", 
            task.id.unwrap_or(0),
            priority,
            task.project.as_deref().unwrap_or(""),
            task.description
        );
    }

    println!("\n4. Integration test complete! ✅");
    println!("LazyTask is ready to manage your Taskwarrior tasks.");

    Ok(())
}

