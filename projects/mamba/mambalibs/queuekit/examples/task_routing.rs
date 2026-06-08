//! Example: Task Routing Integration
//!
//! This example demonstrates how to integrate the Router with TaskRegistry
//! for automatic task queue routing.

use queuekit::{
    Task, TaskContext, TaskOutcome, TaskRegistry, RouterConfig,
};
use async_trait::async_trait;

// Define some example tasks
struct MathTask;
struct EmailTask;
struct UrgentTask;

#[async_trait]
impl Task for MathTask {
    fn name(&self) -> &'static str {
        "math.add"
    }

    async fn execute(&self, _ctx: TaskContext, args: serde_json::Value) -> TaskOutcome {
        println!("Executing math task with args: {}", args);
        TaskOutcome::Success(args)
    }
}

#[async_trait]
impl Task for EmailTask {
    fn name(&self) -> &'static str {
        "email.send"
    }

    async fn execute(&self, _ctx: TaskContext, args: serde_json::Value) -> TaskOutcome {
        println!("Executing email task with args: {}", args);
        TaskOutcome::Success(args)
    }
}

#[async_trait]
impl Task for UrgentTask {
    fn name(&self) -> &'static str {
        "urgent.backup"
    }

    async fn execute(&self, _ctx: TaskContext, args: serde_json::Value) -> TaskOutcome {
        println!("Executing urgent task with args: {}", args);
        TaskOutcome::Success(args)
    }
}

fn main() {
    println!("=== Task Routing Integration Example ===\n");

    // Step 1: Create a router with various routing rules
    println!("Step 1: Creating router with rules...");
    let router = RouterConfig::new()
        // Exact match routes
        .route("math.add", "math-workers")

        // Glob pattern routes
        .route_glob("email.*", "email-workers")
        .route_glob("urgent.*", "high-priority")
        .route_glob("analytics.*", "analytics-workers")

        // Custom routing function based on task arguments
        .route_fn("priority_router", |_task_name, args| {
            if let Some(priority) = args.get("priority").and_then(|v| v.as_str()) {
                match priority {
                    "high" => return Some("high-priority".to_string()),
                    "low" => return Some("low-priority".to_string()),
                    _ => {}
                }
            }
            None
        })

        // Default queue for unmatched tasks
        .default_queue("default-workers")
        .build();

    println!("  Router created with 4 static routes + custom function\n");

    // Step 2: Create registry with router
    println!("Step 2: Creating task registry with router...");
    let registry = TaskRegistry::new().with_router(router);
    println!("  Registry created and router attached\n");

    // Step 3: Register tasks
    println!("Step 3: Registering tasks...");
    registry.register(MathTask);
    registry.register(EmailTask);
    registry.register(UrgentTask);
    println!("  Registered {} tasks\n", registry.len());

    // Step 4: Demonstrate routing
    println!("Step 4: Testing automatic routing:\n");

    // Test exact match
    let queue = registry.route_task("math.add", &serde_json::json!({}));
    println!("  math.add → {}", queue);
    assert_eq!(queue, "math-workers");

    // Test glob pattern
    let queue = registry.route_task("email.send", &serde_json::json!({}));
    println!("  email.send → {}", queue);
    assert_eq!(queue, "email-workers");

    let queue = registry.route_task("email.receive", &serde_json::json!({}));
    println!("  email.receive → {}", queue);
    assert_eq!(queue, "email-workers");

    // Test urgent pattern
    let queue = registry.route_task("urgent.backup", &serde_json::json!({}));
    println!("  urgent.backup → {}", queue);
    assert_eq!(queue, "high-priority");

    // Test custom routing based on args
    let queue = registry.route_task(
        "process_data",
        &serde_json::json!({"priority": "high"}),
    );
    println!("  process_data (priority: high) → {}", queue);
    assert_eq!(queue, "high-priority");

    let queue = registry.route_task(
        "process_data",
        &serde_json::json!({"priority": "low"}),
    );
    println!("  process_data (priority: low) → {}", queue);
    assert_eq!(queue, "low-priority");

    // Test default queue
    let queue = registry.route_task("unknown_task", &serde_json::json!({}));
    println!("  unknown_task → {}", queue);
    assert_eq!(queue, "default-workers");

    println!("\n=== All routing tests passed! ===");

    // Step 5: Show registry info
    println!("\nRegistry Statistics:");
    println!("  Total registered tasks: {}", registry.len());
    println!("  Has router: {}", registry.router().is_some());
    println!("  Registered task names: {:?}", registry.list());
}
