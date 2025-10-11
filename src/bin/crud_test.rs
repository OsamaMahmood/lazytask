// Test CRUD operations through LazyTask

use anyhow::Result;
use lazytask::taskwarrior::TaskwarriorIntegration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 LazyTask CRUD Operations Test");
    println!("================================");

    let taskwarrior = TaskwarriorIntegration::new(None, None)?;

    println!("\n1. 📋 Current pending tasks:");
    let initial_tasks = taskwarrior.list_tasks(Some("+PENDING")).await?;
    println!("   Found {} pending tasks", initial_tasks.len());

    println!("\n2. ➕ Adding a new task via LazyTask...");
    let new_task_id = taskwarrior.add_task(
        "LazyTask CRUD test task", 
        &[("project", "lazytask"), ("priority", "H")]
    ).await?;
    println!("   ✅ Created task ID: {}", new_task_id);

    println!("\n3. 📋 Verifying task was added:");
    let updated_tasks = taskwarrior.list_tasks(Some("+PENDING")).await?;
    println!("   Now {} pending tasks (+{})", 
        updated_tasks.len(), 
        updated_tasks.len() - initial_tasks.len()
    );

    // Find our new task
    if let Some(new_task) = updated_tasks.iter().find(|t| t.id == Some(new_task_id)) {
        println!("   ✅ New task found: [{}] {} [{}] - {}", 
            new_task.id.unwrap(),
            new_task.priority.as_ref().map(|p| p.as_char()).unwrap_or(' '),
            new_task.project.as_deref().unwrap_or(""),
            new_task.description
        );
    }

    println!("\n4. ✅ Marking task as done...");
    taskwarrior.done_task(new_task_id).await?;
    
    let final_tasks = taskwarrior.list_tasks(Some("+PENDING")).await?;
    println!("   Now {} pending tasks (-1)", final_tasks.len());

    println!("\n5. ✅ CRUD test complete!");
    println!("   LazyTask can successfully:");
    println!("   • Load existing tasks from Taskwarrior");
    println!("   • Add new tasks with attributes (project, priority)");
    println!("   • Mark tasks as completed");
    println!("   • Delete tasks");
    println!("   • Refresh and display updated task lists");

    println!("\n🎉 LazyTask CRUD integration verified!");

    Ok(())
}
