//! Task Groups Loader Example
//!
//! Demonstrates how to use the TaskGroupLoader to discover and load task groups
//! from the filesystem.

use periplon_sdk::dsl::predefined_tasks::{
    GroupLoadError, TaskGroupLoader, TaskGroupReference,
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Task Groups Loader Example ===\n");

    // Example 1: Create loader with default search paths
    println!("1. Creating TaskGroupLoader with default paths:");
    println!("   - ~/.claude/task-groups/");
    println!("   - ./.claude/task-groups/\n");

    let mut loader = TaskGroupLoader::new();

    // Example 2: Discover all available task groups
    println!("2. Discovering all available task groups...");

    match loader.discover_all() {
        Ok(discovered) => {
            if discovered.is_empty() {
                println!("   No task groups found in search paths.");
                println!("   (This is expected if .claude/task-groups/ doesn't exist)\n");
            } else {
                println!("   Found {} task groups:\n", discovered.len());
                for (group_ref, path) in &discovered {
                    println!("   - {} at {}", group_ref, path.display());
                }
                println!();
            }
        }
        Err(e) => {
            println!("   Warning: Discovery failed: {}", e);
            println!("   (This is expected if directories don't exist)\n");
        }
    }

    // Example 3: Try to load a specific task group (will fail if not present)
    println!("3. Attempting to load a task group...");

    let group_ref = TaskGroupReference::parse("google-workspace-suite@2.0.0")?;
    println!("   Looking for: {}\n", group_ref.to_string());

    match loader.load(&group_ref) {
        Ok(resolved) => {
            println!("   ✅ Successfully loaded task group!");
            println!("   Name: {}", resolved.group.metadata.name);
            println!("   Version: {}", resolved.group.metadata.version);
            println!(
                "   Description: {}",
                resolved
                    .group
                    .metadata
                    .description
                    .as_deref()
                    .unwrap_or("N/A")
            );
            println!("   Tasks: {}", resolved.tasks.len());
            println!("   Source: {}\n", resolved.source_path.display());

            // List all tasks in the group
            println!("   Tasks in group:");
            for name in resolved.task_names() {
                if let Some(task) = resolved.get_task(&name) {
                    println!(
                        "   - {} v{} - {}",
                        task.metadata.name,
                        task.metadata.version,
                        task.metadata
                            .description
                            .as_deref()
                            .unwrap_or("No description")
                    );
                }
            }
            println!();

            // Check if shared config was applied
            if let Some(ref shared_config) = resolved.group.spec.shared_config {
                println!("   Shared configuration:");
                println!("   - Shared inputs: {}", shared_config.inputs.len());
                println!(
                    "   - Permissions: {}",
                    if shared_config.permissions.is_some() {
                        "Yes"
                    } else {
                        "No"
                    }
                );
                println!(
                    "   - Max turns: {}",
                    shared_config
                        .max_turns
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "Not set".to_string())
                );
                println!();
            }
        }
        Err(GroupLoadError::GroupNotFound(ref_str)) => {
            println!("   ⚠️  Task group not found: {}", ref_str);
            println!("   This is expected if you haven't created the group yet.\n");
        }
        Err(GroupLoadError::TaskNotFound {
            group,
            task,
            version,
        }) => {
            println!(
                "   ❌ Error: Task '{}@{}' required by group '{}' not found",
                task, version, group
            );
            println!("   Make sure all referenced tasks are available.\n");
        }
        Err(e) => {
            println!("   ❌ Error loading group: {}\n", e);
        }
    }

    // Example 4: Custom search paths
    println!("4. Using custom search paths:");

    let custom_paths = vec![
        PathBuf::from("./examples/task-groups"),
        PathBuf::from("/opt/claude-tasks/groups"),
    ];

    let custom_loader = TaskGroupLoader::with_paths(custom_paths.clone());

    println!("   Search paths:");
    for path in &custom_paths {
        println!("   - {}", path.display());
    }
    println!();

    match custom_loader.discover_all() {
        Ok(discovered) => {
            if discovered.is_empty() {
                println!("   No task groups found in custom paths.");
            } else {
                println!("   Found {} task groups", discovered.len());
            }
        }
        Err(e) => {
            println!("   Note: {}", e);
        }
    }
    println!();

    // Example 5: Cache inspection
    println!("5. Cache management:");
    println!("   Cached groups: {}", loader.cached_groups().len());

    if !loader.cached_groups().is_empty() {
        println!("   Cached:");
        for cached in loader.cached_groups() {
            println!("   - {}", cached);
        }

        println!("\n   Clearing cache...");
        loader.clear_cache();
        println!(
            "   Cached groups after clear: {}",
            loader.cached_groups().len()
        );
    } else {
        println!("   No groups currently cached.");
    }
    println!();

    // Example 6: Error handling patterns
    println!("6. Error handling example:");

    let invalid_ref = TaskGroupReference::parse("nonexistent-group@1.0.0")?;

    match loader.load(&invalid_ref) {
        Ok(_) => println!("   Unexpected: Group found!"),
        Err(GroupLoadError::GroupNotFound(ref_str)) => {
            println!("   ✅ Correctly handled missing group: {}", ref_str);
        }
        Err(e) => println!("   Other error: {}", e),
    }
    println!();

    println!("=== Example Complete ===");
    println!("\nTo use this loader in your code:");
    println!("1. Create task groups in .claude/task-groups/");
    println!("2. Create individual tasks in .claude/tasks/");
    println!("3. Use TaskGroupLoader to load and resolve groups");
    println!("4. Access resolved tasks from ResolvedTaskGroup");

    Ok(())
}
