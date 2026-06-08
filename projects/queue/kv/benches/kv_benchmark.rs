//! Comprehensive benchmark suite for KV store engine
//!
//! Tests:
//! - Pure engine performance (no network)
//! - Concurrency and lock contention
//! - Memory usage and TTL cleanup
//! - Scalability across different data sizes

use cclab_kv::engine::KvEngine;
use cclab_kv::types::{KvKey, KvValue};
use qc::benchmark::{
    Benchmarker, BenchmarkConfig, BenchmarkReport, BenchmarkReportGroup, BenchmarkEnvironment,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Helper to create a key for benchmarks
fn make_key(id: usize) -> KvKey {
    KvKey::new(format!("bench_key_{}", id)).unwrap()
}

/// Helper to create a string value
fn make_string_value(id: usize) -> KvValue {
    KvValue::String(format!("value_{}", id))
}

/// Benchmark: Single-threaded SET throughput
fn bench_set_single_threaded(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = KvEngine::new();
    let mut counter = 0;

    benchmarker.run("set_single_thread", || {
        let key = make_key(counter);
        let value = make_string_value(counter);
        engine.set(&key, value, None);
        counter += 1;
    })
}

/// Benchmark: Single-threaded GET throughput
fn bench_get_single_threaded(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = KvEngine::new();

    // Pre-populate with 10K entries
    for i in 0..10_000 {
        let key = make_key(i);
        let value = make_string_value(i);
        engine.set(&key, value, None);
    }

    let mut counter = 0;
    benchmarker.run("get_single_thread", || {
        let key = make_key(counter % 10_000);
        let _ = engine.get(&key);
        counter += 1;
    })
}

/// Benchmark: Mixed workload (50% GET, 50% SET)
fn bench_mixed_workload(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = KvEngine::new();

    // Pre-populate
    for i in 0..1_000 {
        let key = make_key(i);
        let value = make_string_value(i);
        engine.set(&key, value, None);
    }

    let mut counter = 0;
    benchmarker.run("mixed_50_50", || {
        if counter % 2 == 0 {
            // GET operation
            let key = make_key(counter % 1_000);
            let _ = engine.get(&key);
        } else {
            // SET operation
            let key = make_key(counter % 1_000);
            let value = make_string_value(counter);
            engine.set(&key, value, None);
        }
        counter += 1;
    })
}

/// Benchmark: INCR atomic operations
fn bench_incr_operations(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = KvEngine::new();
    let key = KvKey::new("counter").unwrap();

    // Initialize counter
    engine.set(&key, KvValue::Int(0), None);

    benchmarker.run("incr_atomic", || {
        let _ = engine.incr(&key, 1);
    })
}

/// Benchmark: DECR atomic operations
fn bench_decr_operations(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = KvEngine::new();
    let key = KvKey::new("counter").unwrap();

    // Initialize counter
    engine.set(&key, KvValue::Int(1_000_000), None);

    benchmarker.run("decr_atomic", || {
        let _ = engine.decr(&key, 1);
    })
}

/// Benchmark: Multi-threaded SET with 2 threads
fn bench_set_concurrent_2_threads(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let config = benchmarker.config();
    let total_ops = (config.iterations * config.rounds) as usize;
    let ops_per_thread = total_ops / 2;

    benchmarker.run("set_concurrent_2t", || {
        let engine = Arc::new(KvEngine::new());
        let mut handles = vec![];

        for thread_id in 0..2 {
            let engine = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let key = make_key(thread_id * ops_per_thread + i);
                    let value = make_string_value(i);
                    engine.set(&key, value, None);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    })
}

/// Benchmark: Multi-threaded SET with 4 threads
fn bench_set_concurrent_4_threads(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let config = benchmarker.config();
    let total_ops = (config.iterations * config.rounds) as usize;
    let ops_per_thread = total_ops / 4;

    benchmarker.run("set_concurrent_4t", || {
        let engine = Arc::new(KvEngine::new());
        let mut handles = vec![];

        for thread_id in 0..4 {
            let engine = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let key = make_key(thread_id * ops_per_thread + i);
                    let value = make_string_value(i);
                    engine.set(&key, value, None);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    })
}

/// Benchmark: Multi-threaded SET with 8 threads
fn bench_set_concurrent_8_threads(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let config = benchmarker.config();
    let total_ops = (config.iterations * config.rounds) as usize;
    let ops_per_thread = total_ops / 8;

    benchmarker.run("set_concurrent_8t", || {
        let engine = Arc::new(KvEngine::new());
        let mut handles = vec![];

        for thread_id in 0..8 {
            let engine = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let key = make_key(thread_id * ops_per_thread + i);
                    let value = make_string_value(i);
                    engine.set(&key, value, None);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    })
}

/// Benchmark: Multi-threaded GET with 2 threads
fn bench_get_concurrent_2_threads(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = Arc::new(KvEngine::new());

    // Pre-populate
    for i in 0..10_000 {
        let key = make_key(i);
        let value = make_string_value(i);
        engine.set(&key, value, None);
    }

    let config = benchmarker.config();
    let total_ops = (config.iterations * config.rounds) as usize;
    let ops_per_thread = total_ops / 2;

    benchmarker.run("get_concurrent_2t", || {
        let mut handles = vec![];

        for thread_id in 0..2 {
            let engine = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let key = make_key((thread_id * ops_per_thread + i) % 10_000);
                    let _ = engine.get(&key);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    })
}

/// Benchmark: Multi-threaded GET with 4 threads
fn bench_get_concurrent_4_threads(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = Arc::new(KvEngine::new());

    // Pre-populate
    for i in 0..10_000 {
        let key = make_key(i);
        let value = make_string_value(i);
        engine.set(&key, value, None);
    }

    let config = benchmarker.config();
    let total_ops = (config.iterations * config.rounds) as usize;
    let ops_per_thread = total_ops / 4;

    benchmarker.run("get_concurrent_4t", || {
        let mut handles = vec![];

        for thread_id in 0..4 {
            let engine = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let key = make_key((thread_id * ops_per_thread + i) % 10_000);
                    let _ = engine.get(&key);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    })
}

/// Benchmark: Multi-threaded GET with 8 threads
fn bench_get_concurrent_8_threads(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = Arc::new(KvEngine::new());

    // Pre-populate
    for i in 0..10_000 {
        let key = make_key(i);
        let value = make_string_value(i);
        engine.set(&key, value, None);
    }

    let config = benchmarker.config();
    let total_ops = (config.iterations * config.rounds) as usize;
    let ops_per_thread = total_ops / 8;

    benchmarker.run("get_concurrent_8t", || {
        let mut handles = vec![];

        for thread_id in 0..8 {
            let engine = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let key = make_key((thread_id * ops_per_thread + i) % 10_000);
                    let _ = engine.get(&key);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    })
}

/// Benchmark: Lock contention with concurrent access
fn bench_lock_contention(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let config = benchmarker.config();
    let total_ops = (config.iterations * config.rounds) as usize;
    let ops_per_thread = total_ops / 4;

    benchmarker.run("lock_contention", || {
        let engine = Arc::new(KvEngine::new());
        let lock_key = KvKey::new("shared_lock").unwrap();
        let mut handles = vec![];

        for thread_id in 0..4 {
            let engine = Arc::clone(&engine);
            let lock_key = lock_key.clone();
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let owner = format!("worker-{}-{}", thread_id, i);
                    // Try to acquire lock
                    while !engine.lock(&lock_key, &owner, Duration::from_millis(100)) {
                        // Spin until we get the lock
                        thread::yield_now();
                    }
                    // Do some work
                    let _ = engine.get(&lock_key);
                    // Release lock
                    let _ = engine.unlock(&lock_key, &owner);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    })
}

/// Benchmark: Memory usage with 100K entries
fn bench_memory_100k_entries(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    benchmarker.run("memory_100k_entries", || {
        let engine = KvEngine::new();

        // Insert 100K entries
        for i in 0..100_000 {
            let key = make_key(i);
            let value = make_string_value(i);
            engine.set(&key, value, None);
        }

        // Verify count
        assert_eq!(engine.len(), 100_000);
    })
}

/// Benchmark: TTL cleanup overhead
fn bench_ttl_cleanup(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    benchmarker.run("ttl_cleanup", || {
        let engine = KvEngine::new();

        // Pre-populate with expired entries
        for i in 0..10_000 {
            let key = make_key(i);
            let value = make_string_value(i);
            engine.set(&key, value, Some(Duration::from_millis(1)));
        }

        // Wait for expiration
        thread::sleep(Duration::from_millis(10));

        // Cleanup (this is the timed operation)
        let removed = engine.cleanup_expired();
        assert!(removed > 0);
    })
}

/// Benchmark: Insertion scalability - 1K entries
fn bench_insert_1k_entries(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    benchmarker.run("insert_1k", || {
        let engine = KvEngine::new();
        for i in 0..1_000 {
            let key = make_key(i);
            let value = make_string_value(i);
            engine.set(&key, value, None);
        }
    })
}

/// Benchmark: Insertion scalability - 10K entries
fn bench_insert_10k_entries(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    benchmarker.run("insert_10k", || {
        let engine = KvEngine::new();
        for i in 0..10_000 {
            let key = make_key(i);
            let value = make_string_value(i);
            engine.set(&key, value, None);
        }
    })
}

/// Benchmark: Insertion scalability - 100K entries
fn bench_insert_100k_entries(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    benchmarker.run("insert_100k", || {
        let engine = KvEngine::new();
        for i in 0..100_000 {
            let key = make_key(i);
            let value = make_string_value(i);
            engine.set(&key, value, None);
        }
    })
}

/// Benchmark: Insertion scalability - 1M entries
fn bench_insert_1m_entries(_benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    // Use quick config for this heavy benchmark
    let benchmarker = Benchmarker::new(BenchmarkConfig::new(1, 1, 0));

    benchmarker.run("insert_1m", || {
        let engine = KvEngine::new();
        for i in 0..1_000_000 {
            let key = make_key(i);
            let value = make_string_value(i);
            engine.set(&key, value, None);
        }
    })
}

/// Benchmark: CAS (Compare-And-Swap) operations
fn bench_cas_operations(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = KvEngine::new();
    let key = KvKey::new("cas_key").unwrap();

    // Initialize
    engine.set(&key, KvValue::Int(0), None);

    let mut counter = 0;
    benchmarker.run("cas_operations", || {
        let expected = KvValue::Int(counter);
        let new_value = KvValue::Int(counter + 1);
        let _ = engine.cas(&key, &expected, new_value, None);
        counter += 1;
    })
}

/// Benchmark: SETNX (Set if Not Exists) operations
fn bench_setnx_operations(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = KvEngine::new();
    let mut counter = 0;

    benchmarker.run("setnx_operations", || {
        let key = make_key(counter);
        let value = make_string_value(counter);
        let _ = engine.setnx(&key, value, None);
        counter += 1;
    })
}

/// Benchmark: EXISTS checks
fn bench_exists_checks(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let engine = KvEngine::new();

    // Pre-populate
    for i in 0..10_000 {
        let key = make_key(i);
        let value = make_string_value(i);
        engine.set(&key, value, None);
    }

    let mut counter = 0;
    benchmarker.run("exists_checks", || {
        let key = make_key(counter % 10_000);
        let _ = engine.exists(&key);
        counter += 1;
    })
}

/// Benchmark: DELETE operations
fn bench_delete_operations(benchmarker: &Benchmarker) -> qc::benchmark::BenchmarkResult {
    let config = benchmarker.config();
    let total_ops = (config.iterations * config.rounds) as usize;

    // Pre-populate enough entries
    let engine = KvEngine::new();
    for i in 0..total_ops {
        let key = make_key(i);
        let value = make_string_value(i);
        engine.set(&key, value, None);
    }

    let mut counter = 0;
    benchmarker.run("delete_operations", || {
        let key = make_key(counter);
        let _ = engine.delete(&key);
        counter += 1;
    })
}

fn main() {
    println!("\n=== KV Store Comprehensive Benchmark Suite ===\n");

    // Use thorough configuration for detailed statistics
    let benchmarker = Benchmarker::new(BenchmarkConfig::thorough());

    let mut report = BenchmarkReport::new("KV Store Performance Benchmarks");
    report = report.with_description(
        "Comprehensive performance analysis of the cclab KV storage engine"
    );

    // Set environment info
    let env = BenchmarkEnvironment {
        rust_version: Some(env!("CARGO_PKG_RUST_VERSION").to_string()),
        platform: Some(std::env::consts::OS.to_string()),
        cpu: Some(format!("{} cores", num_cpus::get())),
        hostname: hostname::get().ok().and_then(|h| h.into_string().ok()),
        python_version: None,
    };
    report.set_environment(env);

    // Group 1: Pure Engine Performance
    println!("Running Pure Engine Performance benchmarks...");
    let mut group1 = BenchmarkReportGroup::new("Pure Engine Performance");
    group1 = group1.with_baseline("set_single_thread");

    group1.add_result(bench_set_single_threaded(&benchmarker));
    group1.add_result(bench_get_single_threaded(&benchmarker));
    group1.add_result(bench_mixed_workload(&benchmarker));
    group1.add_result(bench_incr_operations(&benchmarker));
    group1.add_result(bench_decr_operations(&benchmarker));
    group1.add_result(bench_cas_operations(&benchmarker));
    group1.add_result(bench_setnx_operations(&benchmarker));
    group1.add_result(bench_exists_checks(&benchmarker));
    group1.add_result(bench_delete_operations(&benchmarker));

    report.add_group(group1);

    // Group 2: Concurrency - SET operations
    println!("\nRunning Concurrency (SET) benchmarks...");
    let mut group2 = BenchmarkReportGroup::new("Concurrency - SET Operations");
    group2 = group2.with_baseline("set_concurrent_2t");

    group2.add_result(bench_set_concurrent_2_threads(&benchmarker));
    group2.add_result(bench_set_concurrent_4_threads(&benchmarker));
    group2.add_result(bench_set_concurrent_8_threads(&benchmarker));

    report.add_group(group2);

    // Group 3: Concurrency - GET operations
    println!("\nRunning Concurrency (GET) benchmarks...");
    let mut group3 = BenchmarkReportGroup::new("Concurrency - GET Operations");
    group3 = group3.with_baseline("get_concurrent_2t");

    group3.add_result(bench_get_concurrent_2_threads(&benchmarker));
    group3.add_result(bench_get_concurrent_4_threads(&benchmarker));
    group3.add_result(bench_get_concurrent_8_threads(&benchmarker));

    report.add_group(group3);

    // Group 4: Lock Contention
    println!("\nRunning Lock Contention benchmarks...");
    let mut group4 = BenchmarkReportGroup::new("Lock Contention");

    group4.add_result(bench_lock_contention(&benchmarker));

    report.add_group(group4);

    // Group 5: Memory & TTL
    println!("\nRunning Memory & TTL benchmarks...");
    let mut group5 = BenchmarkReportGroup::new("Memory & TTL Management");

    group5.add_result(bench_memory_100k_entries(&benchmarker));
    group5.add_result(bench_ttl_cleanup(&benchmarker));

    report.add_group(group5);

    // Group 6: Scalability
    println!("\nRunning Scalability benchmarks...");
    let mut group6 = BenchmarkReportGroup::new("Insertion Scalability");
    group6 = group6.with_baseline("insert_1k");

    group6.add_result(bench_insert_1k_entries(&benchmarker));
    group6.add_result(bench_insert_10k_entries(&benchmarker));
    group6.add_result(bench_insert_100k_entries(&benchmarker));
    group6.add_result(bench_insert_1m_entries(&benchmarker));

    report.add_group(group6);

    // Print detailed report to console
    println!("\n{}", report.to_console());

    // Generate outputs
    println!("\nGenerating reports...");

    // Write JSON report
    std::fs::write("kv_benchmark_report.json", report.to_json())
        .expect("Failed to write JSON report");
    println!("  JSON report: kv_benchmark_report.json");

    // Write HTML report
    std::fs::write("kv_benchmark_report.html", report.to_html())
        .expect("Failed to write HTML report");
    println!("  HTML report: kv_benchmark_report.html");

    // Write Markdown report
    std::fs::write("kv_benchmark_report.md", report.to_markdown())
        .expect("Failed to write Markdown report");
    println!("  Markdown report: kv_benchmark_report.md");

    println!("\nBenchmark suite completed!");
}
