//! Example: Using metrics with cclab-meteor
//!
//! This example demonstrates how to use Prometheus metrics to monitor task execution.
//!
//! Run with: cargo run --example metrics_example --features metrics

#[cfg(feature = "metrics")]
use cclab_queue::metrics::{METRICS, gather_metrics};

#[cfg(feature = "metrics")]
fn main() {
    println!("=== cclab-meteor Metrics Example ===\n");

    // Simulate task execution
    println!("Recording task metrics...");

    // Task 1: Successful execution
    METRICS.record_published("process_order", "orders");
    METRICS.record_started("process_order", "orders");
    std::thread::sleep(std::time::Duration::from_millis(100));
    METRICS.record_completed("process_order", "orders", 0.1, true);

    // Task 2: Failed execution with retry
    METRICS.record_published("send_email", "notifications");
    METRICS.record_started("send_email", "notifications");
    METRICS.record_retry("send_email", "notifications");
    METRICS.record_failure("send_email", "notifications", "timeout");

    // Task 3: Another successful execution
    METRICS.record_published("generate_report", "reports");
    METRICS.record_started("generate_report", "reports");
    std::thread::sleep(std::time::Duration::from_millis(200));
    METRICS.record_completed("generate_report", "reports", 0.2, true);

    // Gather and display metrics
    println!("\n--- Prometheus Metrics Output ---\n");
    let metrics = gather_metrics();
    println!("{}", metrics);

    println!("\n=== Metrics recorded successfully ===");
    println!("You can expose these metrics on an HTTP endpoint for Prometheus to scrape.");
}

#[cfg(not(feature = "metrics"))]
fn main() {
    println!("This example requires the 'metrics' feature.");
    println!("Run with: cargo run --example metrics_example --features metrics");
}
