#![cfg(test)]

/// Performance comparison tests: Mamba JIT vs CPython 3.12.
///
/// Each test runs a Python snippet through both the Mamba JIT pipeline and
/// CPython 3.12 (via subprocess), prints wall-clock timings, and computes
/// a speedup ratio. These are NOT correctness tests — they measure relative
/// performance to track regressions and guide optimisation.
///
/// Run: `cargo test -p mamba --test perf_comparison_tests -- --nocapture`
///
/// The `--nocapture` flag is important: without it the comparison table is
/// swallowed by the test harness.
///
/// Output-mismatch handling (closes #2563): a benchmark whose Mamba output
/// disagrees with CPython hard-fails the test. A wrong-output run cannot
/// be trusted as a speedup — fast for the wrong answer is no speedup at
/// all. The assertion is centralized in `assert_all_outputs_match` so a
/// dedicated regression test (`mismatch_aborts_perf_comparison_2563`) can
/// pin the contract without re-running the whole bench corpus.
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

struct PerfResult {
    name: &'static str,
    src: &'static str,
    mamba_us: f64,
    cpython_us: f64,
    speedup: f64,
    output_match: bool,
    mamba_out: String,
    cpython_out: String,
}

/// Run source through Mamba JIT, return (elapsed, captured_output).
fn run_mamba(src: &str, iters: u32) -> (Duration, String) {
    let _guard = JIT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    cleanup_all_runtime_state();

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

    // Warm-up + capture output
    let prev = begin_capture();
    let _ = main_fn();
    cleanup_all_runtime_state();
    let last_output = end_capture(prev);

    // Timed runs — measure only JIT execution, cleanup between runs to
    // avoid state leaking (globals, closures, iterators).
    let mut total_exec = Duration::ZERO;
    for _ in 0..iters {
        let prev = begin_capture();
        let t0 = Instant::now();
        let _ = main_fn();
        total_exec += t0.elapsed();
        cleanup_all_runtime_state();
        end_capture(prev);
    }
    (total_exec, last_output)
}

/// Run source through CPython 3.12 subprocess, return (elapsed, stdout).
fn run_cpython(src: &str, iters: u32) -> Option<(Duration, String)> {
    // Build a self-timing script. The benchmark code's output is suppressed
    // during timed iterations by redirecting stdout; only the final iteration
    // captures output for correctness comparison.
    let timer_script = format!(
        r#"import time, sys, os, io
# Compile once
__src = {src:?}
# Warm-up (unredirected)
__devnull = open(os.devnull, 'w')
__real = sys.stdout
sys.stdout = __devnull
exec(__src)
sys.stdout = __real
# Timed iterations (output suppressed)
sys.stdout = __devnull
__start = time.perf_counter_ns()
for __i in range({iters}):
    exec(__src)
__elapsed = time.perf_counter_ns() - __start
sys.stdout = __real
__devnull.close()
# Capture one final run for output comparison
__buf = io.StringIO()
sys.stdout = __buf
exec(__src)
sys.stdout = __real
# Print elapsed_ns on first line, then the captured output
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

/// Run a single benchmark and return the result row.
fn bench(name: &'static str, src: &'static str, iters: u32) -> PerfResult {
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

    PerfResult {
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

/// Acceptance #1 + #3 (closes #2563): hard-fail when ANY benchmark's
/// output disagrees with CPython. The panic message names every
/// mismatched fixture, includes a compact preview of the disagreeing
/// outputs, and quotes the Python source snippet ("benchmark command")
/// so a worker can reproduce without re-running the whole corpus.
fn assert_all_outputs_match(results: &[PerfResult]) {
    let mismatched: Vec<&PerfResult> = results.iter().filter(|r| !r.output_match).collect();
    if mismatched.is_empty() {
        return;
    }
    let mut msg = String::from(
        "perf_comparison output mismatch — wrong-output speedups are not \
         valid speedups (#2563):\n",
    );
    for r in &mismatched {
        // Compact, deterministic preview: trim + cap at 80 chars per side.
        let trim80 = |s: &str| -> String {
            let t = s.trim();
            if t.chars().count() > 80 {
                let head: String = t.chars().take(77).collect();
                format!("{head}...")
            } else {
                t.to_string()
            }
        };
        msg.push_str(&format!(
            "  fixture={} command=<<<{}>>>\n    mamba   = {:?}\n    cpython = {:?}\n",
            r.name,
            r.src.trim(),
            trim80(&r.mamba_out),
            trim80(&r.cpython_out),
        ));
    }
    panic!("{msg}");
}

fn print_table(results: &[PerfResult]) {
    println!();
    println!(
        "┌{:─<34}┬{:─<16}┬{:─<16}┬{:─<12}┬{:─<9}┐",
        "", "", "", "", ""
    );
    println!(
        "│ {:<32} │ {:>14} │ {:>14} │ {:>10} │ {:<7} │",
        "Benchmark", "Mamba (us/op)", "Py3.12 (us/op)", "Speedup", "Match"
    );
    println!(
        "├{:─<34}┼{:─<16}┼{:─<16}┼{:─<12}┼{:─<9}┤",
        "", "", "", "", ""
    );
    for r in results {
        let match_str = if r.output_match { "OK" } else { "DIFF" };
        println!(
            "│ {:<32} │ {:>14.1} │ {:>14.1} │ {:>9.1}x │ {:<7} │",
            r.name, r.mamba_us, r.cpython_us, r.speedup, match_str
        );
    }
    println!(
        "└{:─<34}┴{:─<16}┴{:─<16}┴{:─<12}┴{:─<9}┘",
        "", "", "", "", ""
    );
    println!();
}

// ── The benchmark suite ────────────────────────────────────────────────────

#[test]
fn perf_comparison_mamba_vs_py312() {
    let results = vec![
        // ── Numeric ──
        bench(
            "int_sum_1m_typed",
            r#"
total: int = 0
i: int = 0
while i < 1000000:
    total = total + i
    i = i + 1
"#,
            5,
        ),
        bench(
            "int_sum_1m_untyped",
            r#"
total = 0
i = 0
while i < 1000000:
    total = total + i
    i = i + 1
"#,
            5,
        ),
        bench(
            "int_mul_factorial_20",
            r#"
result: int = 1
i: int = 1
while i <= 20:
    result = result * i
    i = i + 1
"#,
            200,
        ),
        // ── Recursion ──
        bench(
            "fib_25_typed",
            r#"
def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)
print(fib(25))
"#,
            5,
        ),
        bench(
            "fib_25_untyped",
            r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)
print(fib(25))
"#,
            5,
        ),
        bench(
            "factorial_15_typed",
            r#"
def fact(n: int) -> int:
    if n <= 1:
        return 1
    return n * fact(n - 1)
print(fact(15))
"#,
            200,
        ),
        // ── Range loop (JIT-specialized native counter) ──
        bench(
            "range_sum_1m_typed",
            r#"
total: int = 0
for i in range(1000000):
    total = total + i
print(total)
"#,
            5,
        ),
        bench(
            "range_loop_noop_1m",
            r#"
x: int = 0
for i in range(1000000):
    x = x + 1
print(x)
"#,
            5,
        ),
        // ── Collections ──
        bench(
            "list_comprehension_10k",
            r#"
result = [i * i for i in range(10000)]
print(len(result))
"#,
            20,
        ),
        bench(
            "dict_build_1k",
            r#"
d = {}
i = 0
while i < 1000:
    d[i] = i * i
    i = i + 1
print(len(d))
"#,
            50,
        ),
        bench(
            "list_sort_1k",
            r#"
data = list(range(1000, 0, -1))
sorted_data = sorted(data)
print(sorted_data[0], sorted_data[-1])
"#,
            50,
        ),
        // ── Generators ──
        bench(
            "generator_sum_10k",
            r#"
def gen(n):
    i = 0
    while i < n:
        yield i
        i = i + 1

total = 0
for x in gen(10000):
    total = total + x
print(total)
"#,
            10,
        ),
        // ── String ops ──
        bench(
            "str_join_1k",
            r#"
parts = []
i = 0
while i < 1000:
    parts.append("x")
    i = i + 1
result = ",".join(parts)
print(len(result))
"#,
            20,
        ),
        // ── Closures / higher-order ──
        bench(
            "closure_counter_10k",
            r#"
def make_counter():
    count = 0
    def inc():
        nonlocal count
        count = count + 1
        return count
    return inc

c = make_counter()
i = 0
while i < 10000:
    c()
    i = i + 1
print(c())
"#,
            10,
        ),
        // ── Class ──
        bench(
            "class_instance_10k",
            r#"
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def mag(self):
        return self.x * self.x + self.y * self.y

total = 0
i = 0
while i < 10000:
    p = Point(i, i + 1)
    total = total + p.mag()
    i = i + 1
print(total)
"#,
            5,
        ),
        // ── Exception handling ──
        bench(
            "try_except_loop_10k",
            r#"
total = 0
i = 0
while i < 10000:
    try:
        total = total + i
    except Exception:
        pass
    i = i + 1
print(total)
"#,
            10,
        ),
    ];

    print_table(&results);

    // Print summary stats BEFORE asserting on output mismatch so the
    // human-facing table survives even when the assertion fires (timing
    // numbers can still be useful for triage even when output is wrong).
    let avg_speedup: f64 = results.iter().map(|r| r.speedup).sum::<f64>() / results.len() as f64;
    println!("Average speedup vs CPython 3.12: {avg_speedup:.1}x");
    println!();

    // Acceptance (closes #2563): mismatched output fails the test BEFORE
    // speedup is accepted. Matching output preserves the existing timing
    // flow (the table above is already printed).
    assert_all_outputs_match(&results);
}

// ── Regression: mismatch handling (closes #2563) ───────────────────────────

/// Acceptance #1: a mismatched fixture must hard-fail. Build a synthetic
/// PerfResult marked as mismatched and verify `assert_all_outputs_match`
/// panics with a message that names the fixture and benchmark command.
#[test]
#[should_panic(expected = "fixture=synthetic_mismatch")]
fn mismatch_aborts_perf_comparison_2563() {
    let synthetic = PerfResult {
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

/// Acceptance #2: matching output preserves the existing timing flow —
/// the assertion is a no-op when every result reports `output_match = true`.
#[test]
fn matching_outputs_do_not_panic_2563() {
    let synthetic = PerfResult {
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

/// Acceptance #3: failure names the fixture AND benchmark command. Trap
/// the panic message and inspect it directly so we know the preview is
/// compact, deterministic, and contains both fields.
#[test]
fn mismatch_message_names_fixture_and_command_2563() {
    let synthetic = PerfResult {
        name: "fixture_alpha",
        src: "print(beta)",
        mamba_us: 1.0,
        cpython_us: 1.0,
        speedup: 1.0,
        output_match: false,
        mamba_out: "delta\n".to_string(),
        cpython_out: "gamma\n".to_string(),
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
        msg.contains("fixture=fixture_alpha"),
        "panic must name fixture: {msg}"
    );
    assert!(
        msg.contains("command="),
        "panic must name benchmark command: {msg}"
    );
    assert!(
        msg.contains("print(beta)"),
        "panic must quote the source: {msg}"
    );
    assert!(
        msg.contains("delta"),
        "panic must show mamba output preview: {msg}"
    );
    assert!(
        msg.contains("gamma"),
        "panic must show cpython output preview: {msg}"
    );
    assert!(
        msg.contains("#2563"),
        "panic must cite the tracking issue: {msg}"
    );
}
