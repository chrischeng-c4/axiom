//! Diagnostic tool to analyze lock contention and shard distribution
//!
//! This benchmark helps understand why multi-threaded performance degrades

use cclab_kv::engine::KvEngine;
use cclab_kv::types::{KvKey, KvValue};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn make_key(id: usize) -> KvKey {
    KvKey::new(format!("bench_key_{}", id)).unwrap()
}

fn analyze_shard_distribution() {
    println!("\n=== Shard Distribution Analysis ===\n");

    let _engine = KvEngine::new();
    let num_keys = 10_000;

    // Track which shard each key goes to
    let mut shard_counts: HashMap<usize, usize> = HashMap::new();

    // Use reflection to get shard indices (simplified - we'll hash ourselves)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    for i in 0..num_keys {
        let key = make_key(i);
        let mut hasher = DefaultHasher::new();
        key.as_str().hash(&mut hasher);
        let hash = hasher.finish();
        let shard_idx = hash as usize % 256;

        *shard_counts.entry(shard_idx).or_insert(0) += 1;
    }

    // Calculate distribution statistics
    let mut counts: Vec<usize> = shard_counts.values().copied().collect();
    counts.sort();

    let min = counts.iter().min().unwrap_or(&0);
    let max = counts.iter().max().unwrap_or(&0);
    let avg = num_keys / 256;
    let median = if !counts.is_empty() {
        counts[counts.len() / 2]
    } else {
        0
    };

    println!("Keys: {}", num_keys);
    println!("Shards: 256");
    println!("Non-empty shards: {}", shard_counts.len());
    println!("\nDistribution:");
    println!("  Min keys/shard: {}", min);
    println!("  Max keys/shard: {}", max);
    println!("  Avg keys/shard: {}", avg);
    println!("  Median keys/shard: {}", median);
    println!("  Std deviation: {:.2}", calculate_stddev(&counts, avg as f64));

    // Show top 10 most loaded shards
    let mut sorted_shards: Vec<_> = shard_counts.iter().collect();
    sorted_shards.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    println!("\nTop 10 most loaded shards:");
    for (shard_idx, count) in sorted_shards.iter().take(10) {
        println!("  Shard {}: {} keys", shard_idx, count);
    }
}

fn calculate_stddev(values: &[usize], mean: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let variance = values.iter()
        .map(|&v| {
            let diff = v as f64 - mean;
            diff * diff
        })
        .sum::<f64>() / values.len() as f64;

    variance.sqrt()
}

fn benchmark_concurrent_same_keys() {
    println!("\n=== Concurrent Access to SAME Keys (High Contention) ===\n");

    let engine = Arc::new(KvEngine::new());

    // Pre-populate 100 keys
    for i in 0..100 {
        let key = make_key(i);
        let _ = engine.set(&key, KvValue::Int(0), None);
    }

    for num_threads in [1, 2, 4, 8] {
        let ops_per_thread = 50_000;

        let start = Instant::now();
        let mut handles = vec![];

        for _t in 0..num_threads {
            let engine = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    // All threads access same 100 keys = HIGH CONTENTION
                    let key = make_key(i % 100);
                    let _ = engine.set(&key, KvValue::Int(i as i64), None);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let total_ops = num_threads * ops_per_thread;
        let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();

        println!("{} threads: {:>10.0} ops/sec ({:.3}s for {} ops)",
                 num_threads, ops_per_sec, elapsed.as_secs_f64(), total_ops);
    }
}

fn benchmark_concurrent_different_keys() {
    println!("\n=== Concurrent Access to DIFFERENT Keys (Low Contention) ===\n");

    let engine = Arc::new(KvEngine::new());

    for num_threads in [1, 2, 4, 8] {
        let ops_per_thread = 50_000;

        let start = Instant::now();
        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let engine = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    // Each thread uses unique keys = LOW CONTENTION
                    let key = make_key(thread_id * ops_per_thread + i);
                    let _ = engine.set(&key, KvValue::Int(i as i64), None);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let total_ops = num_threads * ops_per_thread;
        let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();

        println!("{} threads: {:>10.0} ops/sec ({:.3}s for {} ops)",
                 num_threads, ops_per_sec, elapsed.as_secs_f64(), total_ops);
    }
}

fn benchmark_thread_overhead() {
    println!("\n=== Thread Spawning Overhead ===\n");

    let engine = Arc::new(KvEngine::new());
    let ops_per_thread = 50_000;

    for num_threads in [1, 2, 4, 8] {
        let iterations = 5;
        let mut total_elapsed = Duration::ZERO;

        for _iter in 0..iterations {
            let start = Instant::now();
            let mut handles = vec![];

            for thread_id in 0..num_threads {
                let engine = Arc::clone(&engine);
                handles.push(thread::spawn(move || {
                    for i in 0..ops_per_thread {
                        let key = make_key(thread_id * ops_per_thread + i);
                        let _ = engine.set(&key, KvValue::Int(i as i64), None);
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }

            total_elapsed += start.elapsed();
        }

        let avg_elapsed = total_elapsed / iterations;
        let total_ops = num_threads * ops_per_thread;
        let ops_per_sec = total_ops as f64 / avg_elapsed.as_secs_f64();

        println!("{} threads: {:>10.0} ops/sec (avg over {} runs)",
                 num_threads, ops_per_sec, iterations);
    }
}

fn benchmark_persistent_threads() {
    println!("\n=== Persistent Thread Pool (No Spawn Overhead) ===\n");

    use std::sync::mpsc;

    let engine = Arc::new(KvEngine::new());

    for num_threads in [1, 2, 4, 8] {
        let ops_per_thread = 50_000;

        // Create persistent threads with channels
        let (tx, rx) = mpsc::channel();
        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let engine = Arc::clone(&engine);
            let tx = tx.clone();

            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let key = make_key(thread_id * ops_per_thread + i);
                    let _ = engine.set(&key, KvValue::Int(i as i64), None);
                }
                tx.send(()).unwrap();
            }));
        }
        drop(tx);

        let start = Instant::now();

        // Wait for all threads to complete
        for _ in 0..num_threads {
            rx.recv().unwrap();
        }

        let elapsed = start.elapsed();

        for handle in handles {
            handle.join().unwrap();
        }

        let total_ops = num_threads * ops_per_thread;
        let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();

        println!("{} threads: {:>10.0} ops/sec ({:.3}s for {} ops)",
                 num_threads, ops_per_sec, elapsed.as_secs_f64(), total_ops);
    }
}

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  KV Store Lock Contention Diagnostic                          ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    analyze_shard_distribution();
    benchmark_concurrent_same_keys();
    benchmark_concurrent_different_keys();
    benchmark_thread_overhead();
    benchmark_persistent_threads();

    println!("\n{}", "=".repeat(70));
}
