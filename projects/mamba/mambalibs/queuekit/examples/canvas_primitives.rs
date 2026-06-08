//! Example demonstrating Canvas primitives (map, starmap, chunks)
//!
//! This example shows how to use Map, Starmap, and Chunks primitives
//! to create parallel task workflows.

use queuekit::*;
use serde_json::json;

fn main() {
    println!("=== Canvas Primitives Example ===\n");

    // Example 1: Map - apply task to each item
    println!("1. Map - Apply 'process_item' to each item:");
    let items = vec![json!(1), json!(2), json!(3), json!(4), json!(5)];
    let map = Map::new("process_item", items.clone());
    println!("   Created map with {} tasks", map.items.len());

    let group = map.to_group();
    println!("   Converted to group with {} tasks", group.tasks.len());
    println!("   Task 0: {} with args: {}", group.tasks[0].task_name, group.tasks[0].args);
    println!();

    // Example 2: Map with helper function
    println!("2. Map using xmap helper:");
    let map2 = xmap("process", vec![json!("a"), json!("b"), json!("c")]);
    let group2 = map2.to_group();
    println!("   Created {} tasks using xmap", group2.tasks.len());
    println!();

    // Example 3: Starmap - apply task with unpacked args
    println!("3. Starmap - Apply 'add' with unpacked args:");
    let tuples = vec![
        vec![json!(1), json!(2)],
        vec![json!(3), json!(4)],
        vec![json!(5), json!(6)],
    ];
    let starmap_obj = Starmap::new("add", tuples);
    let group3 = starmap_obj.to_group();
    println!("   Created {} tasks", group3.tasks.len());
    println!("   Task 0: {} with args: {}", group3.tasks[0].task_name, group3.tasks[0].args);
    println!("   Task 1: {} with args: {}", group3.tasks[1].task_name, group3.tasks[1].args);
    println!();

    // Example 4: Starmap with helper
    println!("4. Starmap using helper:");
    let sm = starmap("multiply", vec![
        vec![json!(2), json!(3)],
        vec![json!(4), json!(5)],
    ]);
    println!("   Created starmap with {} tasks", sm.items.len());
    println!();

    // Example 5: Chunks - split into batches
    println!("5. Chunks - Split 10 items into chunks of 3:");
    let items = vec![json!(1), json!(2), json!(3), json!(4), json!(5),
                     json!(6), json!(7), json!(8), json!(9), json!(10)];
    let chunks_obj = Chunks::new("batch_process", items, 3);
    println!("   Total items: {}", chunks_obj.items.len());
    println!("   Chunk size: {}", chunks_obj.chunk_size);
    println!("   Number of chunks: {}", chunks_obj.num_chunks());

    let group5 = chunks_obj.to_group();
    println!("   Created {} tasks", group5.tasks.len());
    for (i, task) in group5.tasks.iter().enumerate() {
        println!("   Chunk {}: {}", i, task.args);
    }
    println!();

    // Example 6: Chunks with options
    println!("6. Chunks with queue option:");
    let chunks2 = chunks("process", vec![json!(1), json!(2), json!(3), json!(4)], 2)
        .with_options(TaskOptions::new().with_queue("batch"));
    let group6 = chunks2.to_group();
    println!("   Queue: {:?}", group6.options.queue);
    println!("   Task 0 queue: {:?}", group6.tasks[0].options.queue);
    println!();

    // Example 7: Map with options
    println!("7. Map with countdown option:");
    let map3 = Map::new("delayed_task", vec![json!(1), json!(2)])
        .with_options(TaskOptions::new().with_countdown(30));
    let group7 = map3.to_group();
    println!("   Countdown: {:?} seconds", group7.options.countdown);
    println!();

    println!("=== All examples completed ===");
}
