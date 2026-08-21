#![cfg(test)]

/// Performance benchmarks: Mamba JIT vs CPython 3.12 (#1038).
///
/// Five compute-bound benchmarks measuring wall-clock time for both the Mamba
/// JIT pipeline and CPython 3.12 (via subprocess). Each benchmark prints a
/// comparison row; the final table summarises all results.
///
/// Run:
///   cargo test -p mamba --test perf_benchmark_tests --release -- --nocapture
///
/// The `--release` flag is important for meaningful JIT numbers. The
/// `--nocapture` flag is required to see the comparison table.
///
/// Output-mismatch handling (closes #2564): a benchmark whose Mamba output
/// disagrees with CPython hard-fails the test. The check is centralized in
/// `assert_all_outputs_match` and runs AFTER `print_table` so the timing
/// row is preserved as diagnostics. A wrong-output run cannot be trusted
/// as a speedup — fast for the wrong answer is no speedup at all.
use std::process::Command;
use std::time::{Duration, Instant};

use crate::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::lower::{lower_hir_to_mir_with_symbols, lower_module};
use crate::parser;
use crate::runtime::cleanup_all_runtime_state;
use crate::runtime::output::{begin_capture, end_capture};
use crate::source::span::FileId;
use crate::types::TypeChecker;

// ── Helpers ────────────────────────────────────────────────────────────────

struct BenchResult {
    name: &'static str,
    src: &'static str,
    mamba_us: f64,
    cpython_us: f64,
    speedup: f64,
    output_match: bool,
    mamba_out: String,
    cpython_out: String,
}

/// Compile and run `src` through the full Mamba JIT pipeline.
///
/// Returns (total_elapsed across `iters` runs, captured stdout from last run).
fn run_mamba(src: &str, iters: u32) -> (Duration, String) {
    let _guard = JIT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    cleanup_all_runtime_state();

    // Parse -> typecheck -> HIR -> MIR -> JIT codegen
    let module = parser::parse(src, FileId(0)).expect("parse");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    assert!(errors.is_empty(), "type errors: {:?}", errors);

    let hir = lower_module(&module, &checker).expect("HIR");
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new().expect("JIT init");
    let output = backend.codegen(&mir, &checker.tcx).expect("codegen");

    let entry_addr = match output {
        CodegenOutput::Jit { entry } => entry as usize,
        _ => panic!("expected JIT output"),
    };

    let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry_addr) };

    // Warm-up run + capture output for correctness comparison
    let prev = begin_capture();
    let _ = main_fn();
    cleanup_all_runtime_state();
    let last_output = end_capture(prev);

    // Timed runs -- measure only execution, not compilation
    let mut total = Duration::ZERO;
    for _ in 0..iters {
        let prev = begin_capture();
        let t0 = Instant::now();
        let _ = main_fn();
        total += t0.elapsed();
        cleanup_all_runtime_state();
        end_capture(prev);
    }
    (total, last_output)
}

/// Run `src` through CPython 3.12 subprocess with self-timing.
///
/// Returns (total_elapsed across `iters` runs, captured stdout from final run).
/// Returns `None` if python3 is not available or the script fails.
fn run_cpython(src: &str, iters: u32) -> Option<(Duration, String)> {
    let timer_script = format!(
        r#"import time, sys, os, io
__src = {src:?}
__devnull = open(os.devnull, 'w')
__real = sys.stdout
sys.stdout = __devnull
exec(__src)
sys.stdout = __real
sys.stdout = __devnull
__start = time.perf_counter_ns()
for __i in range({iters}):
    exec(__src)
__elapsed = time.perf_counter_ns() - __start
sys.stdout = __real
__devnull.close()
__buf = io.StringIO()
sys.stdout = __buf
exec(__src)
sys.stdout = __real
print(__elapsed)
print(__buf.getvalue(), end='')
"#,
        src = src,
        iters = iters,
    );

    let output = Command::new("python3")
        .arg("-c")
        .arg(&timer_script)
        .output()
        .ok()?;

    if !output.status.success() {
        eprintln!(
            "[cpython stderr] {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let ns: u64 = lines.next()?.trim().parse().ok()?;
    let captured: String = lines.collect::<Vec<_>>().join("\n");
    Some((Duration::from_nanos(ns), captured))
}

/// Run a single benchmark through both runtimes and collect the result.
fn bench(name: &'static str, src: &'static str, iters: u32) -> BenchResult {
    let (mamba_dur, mamba_out) = run_mamba(src, iters);
    let (cpython_dur, cpython_out) =
        run_cpython(src, iters).unwrap_or((Duration::from_secs(999), String::new()));

    let mamba_us = mamba_dur.as_secs_f64() * 1e6 / iters as f64;
    let cpython_us = cpython_dur.as_secs_f64() * 1e6 / iters as f64;
    let speedup = if mamba_us > 0.0 {
        cpython_us / mamba_us
    } else {
        0.0
    };
    let output_match = mamba_out.trim() == cpython_out.trim();

    BenchResult {
        name,
        src,
        mamba_us,
        cpython_us,
        speedup,
        output_match,
        mamba_out,
        cpython_out,
    }
}

/// Acceptance #1 + #3 (closes #2564): hard-fail when ANY benchmark's
/// output diverges from CPython. The panic message names every mismatched
/// benchmark id and includes a compact, deterministic diff of the two
/// outputs (capped at 160 chars per side so the message stays scannable).
fn assert_all_outputs_match(results: &[BenchResult]) {
    let mismatched: Vec<&BenchResult> = results.iter().filter(|r| !r.output_match).collect();
    if mismatched.is_empty() {
        return;
    }
    let preview = |s: &str| -> String {
        let t = s.trim();
        if t.chars().count() > 160 {
            let head: String = t.chars().take(157).collect();
            format!("{head}...")
        } else {
            t.to_string()
        }
    };
    let mut msg = String::from(
        "perf_benchmark output mismatch — wrong-output speedups are not \
         valid speedups (#2564):\n",
    );
    for r in &mismatched {
        msg.push_str(&format!(
            "  benchmark_id={}\n    source = {:?}\n    mamba   = {:?}\n    cpython = {:?}\n",
            r.name,
            r.src.trim(),
            preview(&r.mamba_out),
            preview(&r.cpython_out),
        ));
    }
    panic!("{msg}");
}

fn print_table(results: &[BenchResult]) {
    println!();
    println!(
        "+{:-<34}+{:-<16}+{:-<16}+{:-<12}+{:-<9}+",
        "", "", "", "", ""
    );
    println!(
        "| {:<32} | {:>14} | {:>14} | {:>10} | {:<7} |",
        "Benchmark", "Mamba (us/op)", "Py3.12 (us/op)", "Speedup", "Match"
    );
    println!(
        "+{:-<34}+{:-<16}+{:-<16}+{:-<12}+{:-<9}+",
        "", "", "", "", ""
    );
    for r in results {
        let match_str = if r.output_match { "OK" } else { "DIFF" };
        println!(
            "| {:<32} | {:>14.1} | {:>14.1} | {:>9.1}x | {:<7} |",
            r.name, r.mamba_us, r.cpython_us, r.speedup, match_str
        );
    }
    println!(
        "+{:-<34}+{:-<16}+{:-<16}+{:-<12}+{:-<9}+",
        "", "", "", "", ""
    );
    println!();
}

// ── Benchmark suite ───────────────────────────────────────────────────────

#[test]
fn perf_benchmark_fibonacci_30() {
    let results = vec![bench(
        "fib(30) recursive",
        r#"
def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)
print(fib(30))
"#,
        3,
    )];
    print_table(&results);
    assert_all_outputs_match(&results);
}

#[test]
fn perf_benchmark_list_comprehension_10k() {
    let results = vec![bench(
        "list comprehension 10K",
        r#"
result = [i * i for i in range(10000)]
print(len(result))
"#,
        10,
    )];
    print_table(&results);
    assert_all_outputs_match(&results);
}

#[test]
fn perf_benchmark_dict_operations_10k() {
    let results = vec![bench(
        "dict ops 10K insert+lookup",
        r#"
d = {}
i = 0
while i < 10000:
    d[i] = i * i
    i = i + 1
total = 0
i = 0
while i < 10000:
    total = total + d[i]
    i = i + 1
print(total)
"#,
        5,
    )];
    print_table(&results);
    assert_all_outputs_match(&results);
}

#[test]
fn perf_benchmark_string_concat_10k() {
    let results = vec![bench(
        "string concat 10K",
        r#"
parts = []
i = 0
while i < 10000:
    parts.append("x")
    i = i + 1
result = "".join(parts)
print(len(result))
"#,
        10,
    )];
    print_table(&results);
    assert_all_outputs_match(&results);
}

#[test]
fn perf_benchmark_for_loop_sum_100k() {
    let results = vec![bench(
        "for loop sum 100K",
        r#"
total: int = 0
for i in range(100000):
    total = total + i
print(total)
"#,
        5,
    )];
    print_table(&results);
    assert_all_outputs_match(&results);
}

/// Combined suite: runs all five benchmarks and prints a single summary table.
#[test]
fn perf_benchmark_full_suite() {
    let results = vec![
        bench(
            "fib(30) recursive",
            r#"
def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)
print(fib(30))
"#,
            3,
        ),
        bench(
            "list comprehension 10K",
            r#"
result = [i * i for i in range(10000)]
print(len(result))
"#,
            10,
        ),
        bench(
            "dict ops 10K insert+lookup",
            r#"
d = {}
i = 0
while i < 10000:
    d[i] = i * i
    i = i + 1
total = 0
i = 0
while i < 10000:
    total = total + d[i]
    i = i + 1
print(total)
"#,
            5,
        ),
        bench(
            "string concat 10K",
            r#"
parts = []
i = 0
while i < 10000:
    parts.append("x")
    i = i + 1
result = "".join(parts)
print(len(result))
"#,
            10,
        ),
        bench(
            "for loop sum 100K",
            r#"
total: int = 0
for i in range(100000):
    total = total + i
print(total)
"#,
            5,
        ),
    ];

    print_table(&results);

    // Summary statistics before the assertion so the timing table and
    // average speedup are always visible — even on mismatch.
    let avg_speedup: f64 = results.iter().map(|r| r.speedup).sum::<f64>() / results.len() as f64;
    println!("Average speedup vs CPython 3.12: {avg_speedup:.1}x");

    // Acceptance (closes #2564): mismatched output fails the test before
    // any "PASS" claim. Matching output still proceeds to timing comparison
    // (the table above is already printed).
    assert_all_outputs_match(&results);
    println!();
}

// ── Regression: mismatch handling (closes #2564) ───────────────────────────

/// Acceptance #1: a known output mismatch returns a nonzero test failure.
/// Build a synthetic mismatched BenchResult and verify the centralized
/// assertion panics with the expected benchmark id in the message.
#[test]
#[should_panic(expected = "benchmark_id=synthetic_mismatch")]
fn mismatch_aborts_perf_benchmark_2564() {
    let synthetic = BenchResult {
        name: "synthetic_mismatch",
        src: "print(2 + 2)",
        mamba_us: 1.0,
        cpython_us: 1.0,
        speedup: 1.0,
        output_match: false,
        mamba_out: "5\n".to_string(),
        cpython_out: "4\n".to_string(),
    };
    assert_all_outputs_match(&[synthetic]);
}

/// Acceptance #2: matching output proceeds to the timing comparison —
/// the assertion is a no-op when every result reports `output_match`.
#[test]
fn matching_outputs_proceed_to_timing_2564() {
    let synthetic = BenchResult {
        name: "synthetic_match",
        src: "print(2 + 2)",
        mamba_us: 1.0,
        cpython_us: 1.0,
        speedup: 1.0,
        output_match: true,
        mamba_out: "4\n".to_string(),
        cpython_out: "4\n".to_string(),
    };
    assert_all_outputs_match(&[synthetic]);
}

/// Acceptance #3: failure message includes benchmark id and both outputs
/// (a concise diff). Catch the panic and inspect its payload directly.
#[test]
fn mismatch_message_includes_id_and_both_outputs_2564() {
    let synthetic = BenchResult {
        name: "fixture_delta",
        src: "print(99)",
        mamba_us: 1.0,
        cpython_us: 1.0,
        speedup: 1.0,
        output_match: false,
        mamba_out: "left-side\n".to_string(),
        cpython_out: "right-side\n".to_string(),
    };
    let res = std::panic::catch_unwind(|| {
        assert_all_outputs_match(&[synthetic]);
    });
    let payload = res.expect_err("must panic on mismatch");
    let msg = payload
        .downcast_ref::<String>()
        .map(|s| s.clone())
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        msg.contains("benchmark_id=fixture_delta"),
        "must name id: {msg}"
    );
    assert!(
        msg.contains("left-side"),
        "must include mamba output: {msg}"
    );
    assert!(
        msg.contains("right-side"),
        "must include cpython output: {msg}"
    );
    assert!(msg.contains("#2564"), "must cite the tracking issue: {msg}");
}
