//! Example demonstrating task routing functionality
//!
//! This example shows how to configure and use the Router to direct tasks
//! to different queues based on patterns and custom logic.

use queuekit::routing::RouterConfig;
use serde_json::json;

fn main() {
    println!("=== Task Routing Example ===\n");

    // Example 1: Basic exact match routing
    println!("1. Exact match routing:");
    let router = RouterConfig::new()
        .route("send_email", "email")
        .route("process_payment", "payments")
        .route("generate_report", "reports")
        .default_queue("default")
        .build();

    let tasks = vec![
        ("send_email", json!({"to": "user@example.com"})),
        ("process_payment", json!({"amount": 100})),
        ("generate_report", json!({"type": "monthly"})),
        ("unknown_task", json!({})),
    ];

    for (task_name, args) in &tasks {
        let queue = router.route(task_name, args);
        println!("  {} -> queue: {}", task_name, queue);
    }

    // Example 2: Glob pattern routing
    println!("\n2. Glob pattern routing:");
    let router = RouterConfig::new()
        .route_glob("email.*", "email")
        .route_glob("tasks.math.*", "math")
        .route_glob("tasks.*.urgent", "high-priority")
        .default_queue("default")
        .build();

    let tasks = vec![
        "email.send",
        "email.receive",
        "tasks.math.add",
        "tasks.math.multiply",
        "tasks.email.urgent",
        "tasks.payment.urgent",
        "other.task",
    ];

    for task_name in &tasks {
        let queue = router.route(task_name, &json!({}));
        println!("  {} -> queue: {}", task_name, queue);
    }

    // Example 3: Regex pattern routing
    println!("\n3. Regex pattern routing:");
    let router = RouterConfig::new()
        .route_regex(r"^user_\d+$", "users")
        .route_regex(r"^report_.*_monthly$", "reports")
        .route_regex(r"^(critical|urgent)_", "high-priority")
        .default_queue("default")
        .build();

    let tasks = vec![
        "user_123",
        "user_456",
        "report_sales_monthly",
        "report_usage_monthly",
        "critical_alert",
        "urgent_task",
        "normal_task",
    ];

    for task_name in &tasks {
        let queue = router.route(task_name, &json!({}));
        println!("  {} -> queue: {}", task_name, queue);
    }

    // Example 4: Custom routing function based on arguments
    println!("\n4. Custom routing based on task arguments:");
    let router = RouterConfig::new()
        .route_fn("priority_router", |task_name, args| {
            // Route based on priority argument
            if let Some(priority) = args.get("priority").and_then(|v| v.as_str()) {
                match priority {
                    "high" => return Some("high-priority".to_string()),
                    "low" => return Some("low-priority".to_string()),
                    _ => {}
                }
            }

            // Route based on task name prefix
            if task_name.starts_with("urgent_") {
                return Some("urgent".to_string());
            }

            None
        })
        .default_queue("default")
        .build();

    let tasks = vec![
        ("process_order", json!({"priority": "high"})),
        ("send_newsletter", json!({"priority": "low"})),
        ("urgent_notification", json!({})),
        ("regular_task", json!({})),
    ];

    for (task_name, args) in &tasks {
        let queue = router.route(task_name, args);
        println!("  {} (args: {}) -> queue: {}", task_name, args, queue);
    }

    // Example 5: JSON configuration
    println!("\n5. Loading routes from JSON configuration:");
    let json_config = r#"{
        "routes": [
            {"pattern": "email.*", "queue": "email", "pattern_type": "glob"},
            {"pattern": "payment.*", "queue": "payments", "pattern_type": "glob"},
            {"pattern": "^user_\\d+$", "queue": "users", "pattern_type": "regex"}
        ],
        "default_queue": "worker"
    }"#;

    let config: queuekit::routing::RoutesConfig =
        serde_json::from_str(json_config).expect("Failed to parse JSON");
    let router = config.into_router();

    println!("  Loaded {} routes", router.routes().len());
    println!("  Default queue: {}", router.default_queue());

    let tasks = vec!["email.send", "payment.process", "user_123", "other_task"];
    for task_name in &tasks {
        let queue = router.route(task_name, &json!({}));
        println!("  {} -> queue: {}", task_name, queue);
    }

    // Example 6: Combined routing (custom + patterns)
    println!("\n6. Combined routing strategies:");
    let router = RouterConfig::new()
        // Custom route (checked first)
        .route_fn("vip_router", |task_name, args| {
            if let Some(user_type) = args.get("user_type").and_then(|v| v.as_str()) {
                if user_type == "vip" {
                    return Some("vip".to_string());
                }
            }
            if task_name.contains("premium") {
                return Some("premium".to_string());
            }
            None
        })
        // Pattern routes (checked after custom)
        .route_glob("email.*", "email")
        .route("process_payment", "payments")
        .default_queue("default")
        .build();

    let tasks = vec![
        ("send_email", json!({"user_type": "vip"})),
        ("email.send", json!({})),
        ("premium_feature", json!({})),
        ("process_payment", json!({})),
        ("regular_task", json!({})),
    ];

    for (task_name, args) in &tasks {
        let queue = router.route(task_name, args);
        println!("  {} (args: {}) -> queue: {}", task_name, args, queue);
    }

    println!("\n=== Example Complete ===");
}
