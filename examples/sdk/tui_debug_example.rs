//! TUI Debug Example
//!
//! Demonstrates the full-screen Terminal User Interface for interactive debugging
//! with multiple panes, keyboard shortcuts, and real-time state inspection.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tui_debug_example --features tui
//! ```
use periplon_sdk::dsl::{parse_workflow_file, DSLExecutor, DebugTUI};
use periplon_sdk::error::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== TUI Debug Example ===\n");

    // Load workflow
    let workflow_path = PathBuf::from("examples/workflows/debug_example.yaml");
    println!("📄 Loading workflow: {:?}", workflow_path);

    let workflow = parse_workflow_file(&workflow_path)?;
    println!("✅ Workflow loaded: {}\n", workflow.name);

    // Create executor with debugging enabled
    println!("🐛 Creating executor with debug mode enabled...");
    let executor = DSLExecutor::new(workflow)?.with_debugger();
    println!("✅ Debug mode enabled\n");

    println!("🚀 Launching Debug TUI...\n");
    println!("📝 TUI Features:");
    println!("  - Multi-pane layout (Workflow Tree, Variables, Timeline, REPL)");
    println!("  - Tab / Shift+Tab to switch between panes");
    println!("  - F5: Continue, F10: Step Over, F11: Step Into");
    println!("  - ?: Toggle help overlay");
    println!("  - q: Quit\n");

    // Create and run TUI
    let mut tui = DebugTUI::new()?.with_executor(executor)?;
    tui.run().await?;

    println!("\n✅ TUI session ended");

    Ok(())
}
