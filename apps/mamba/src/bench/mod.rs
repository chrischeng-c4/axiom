use std::process::Command;
/// Mamba benchmark harness (R1).
///
/// Provides micro-benchmark and real-world workload execution for the
/// `mamba bench` CLI command. Measures JIT-compiled Mamba execution time
/// and optionally compares against CPython 3.12 via subprocess invocation.
///
/// # Architecture
///
/// ```text
/// BenchSuite
///   └── Vec<Benchmark>         -- registered workloads
///         ├── name: &str
///         ├── source: &str     -- Mamba/Python source
///         └── kind: BenchKind  -- Numeric | Recursion | Workload
/// BenchRunner
///   ├── run_mamba(bench) → BenchResult
///   └── run_cpython(bench) → Option<BenchResult>
/// BenchReport
///   └── print_table(results)
/// ```
use std::time::{Duration, Instant};

use crate::codegen::cranelift::jit::CraneliftJitBackend;
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::lower::{lower_hir_to_mir_with_symbols, lower_module};
use crate::parser;
use crate::runtime::output::{begin_capture, end_capture};
use crate::source::span::FileId;
use crate::types::TypeChecker;

// ── Benchmark category ──────────────────────────────────────────────────────

/// Category of a benchmark workload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchKind {
    /// Tight numeric loop (e.g. integer summation).
    Numeric,
    /// Recursive function call (e.g. Fibonacci).
    Recursion,
    /// Real-world workload (e.g. string processing, sorting).
    Workload,
}

impl BenchKind {
    pub fn label(self) -> &'static str {
        match self {
            BenchKind::Numeric => "numeric",
            BenchKind::Recursion => "recursion",
            BenchKind::Workload => "workload",
        }
    }
}

// ── Benchmark definition ────────────────────────────────────────────────────

/// A single benchmark: a named Mamba/Python source snippet with a category.
#[derive(Debug, Clone)]
pub struct Benchmark {
    /// Short identifier (no spaces).
    pub name: &'static str,
    /// Mamba/Python source to compile and run.
    pub source: &'static str,
    /// Workload category.
    pub kind: BenchKind,
    /// Number of iterations (outer repeat count for timing).
    pub iters: u32,
}

// ── Result ──────────────────────────────────────────────────────────────────

/// Timing result for one benchmark / one engine.
#[derive(Debug, Clone)]
pub struct BenchResult {
    /// Total wall time across all iterations.
    pub total: Duration,
    /// Number of iterations actually executed.
    pub iters: u32,
}

impl BenchResult {
    /// Mean time per iteration.
    pub fn mean(&self) -> Duration {
        if self.iters == 0 {
            return Duration::ZERO;
        }
        self.total / self.iters
    }
}

// ── Runner ──────────────────────────────────────────────────────────────────

/// Runs a benchmark under Mamba JIT.
pub struct BenchRunner {
    /// Path to the `python3` binary for CPython comparison. `None` skips comparison.
    pub python3_bin: Option<String>,
    /// Path to the `pypy3` binary for PyPy comparison. `None` skips PyPy comparison.
    pub pypy3_bin: Option<String>,
}

impl Default for BenchRunner {
    fn default() -> Self {
        // Auto-detect python3 and pypy3 on PATH.
        let has_python = Command::new("python3").arg("--version").output().is_ok();
        let has_pypy = Command::new("pypy3").arg("--version").output().is_ok();
        Self {
            python3_bin: if has_python {
                Some("python3".to_string())
            } else {
                None
            },
            pypy3_bin: if has_pypy {
                Some("pypy3".to_string())
            } else {
                None
            },
        }
    }
}

impl BenchRunner {
    /// Create a runner that always skips CPython and PyPy comparison.
    pub fn mamba_only() -> Self {
        Self {
            python3_bin: None,
            pypy3_bin: None,
        }
    }

    /// Run `bench` under the Mamba JIT and return timing.
    ///
    /// Compiles once, then executes `bench.iters` times.
    pub fn run_mamba(&self, bench: &Benchmark) -> Result<BenchResult, String> {
        let file_id = FileId(0);

        // Parse
        let module =
            parser::parse(bench.source, file_id).map_err(|e| format!("parse error: {e}"))?;

        // Type check
        let mut checker = TypeChecker::new();
        let errors = checker.check_module(&module);
        if !errors.is_empty() {
            return Err(format!(
                "type errors: {:?}",
                errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
            ));
        }

        // Lower AST → HIR → MIR
        let hir =
            lower_module(&module, &checker).map_err(|errs| format!("HIR error: {:?}", errs))?;
        let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

        // JIT compile
        let mut backend = CraneliftJitBackend::new().map_err(|e| format!("JIT init: {e}"))?;
        let output = backend
            .codegen(&mir, &checker.tcx)
            .map_err(|e| format!("codegen: {e}"))?;

        let entry_addr = match output {
            CodegenOutput::Jit { entry } => entry as usize,
            _ => return Err("expected JIT output".into()),
        };

        // Time `iters` executions
        let iters = bench.iters.max(1);
        let start = Instant::now();
        for _ in 0..iters {
            let prev = begin_capture();
            let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry_addr) };
            let _result = main_fn();
            end_capture(prev); // discard output during bench
        }
        let total = start.elapsed();

        // #1274 probe: scale sweep to detect heap-retention leak.
        // Per-iter cost should be flat across N; if it grows linearly with cumulative
        // iteration count, function-local heap is leaking across calls.
        if std::env::var("MAMBA_BENCH_SCALE_SWEEP").is_ok() {
            let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry_addr) };
            let (c0, t0, _) = crate::runtime::gc::gc_get_stats();
            for &n in &[10u32, 50, 100, 200, 500] {
                let prev = begin_capture();
                let s = Instant::now();
                for _ in 0..n {
                    let _ = main_fn();
                }
                let per = s.elapsed() / n;
                end_capture(prev);
                let (c, t, thr, ac, en, col) = crate::runtime::gc::gc_get_full_stats();
                eprintln!("[scale] {:>22}: N={:>3} → {:?}/iter  tracked={} cycles={} alloc_cnt={}/{} en={} col={}",
                    bench.name, n, per, t, c, ac, thr, en, col);
            }
            let (c1, t1, _) = crate::runtime::gc::gc_get_stats();
            eprintln!(
                "[scale] {:>22}: Δtracked={:+} Δcycles={:+}",
                bench.name,
                (t1 as i64) - (t0 as i64),
                (c1 as i64) - (c0 as i64)
            );
        }

        Ok(BenchResult { total, iters })
    }

    /// Run `bench` under CPython 3.12 and return timing.
    ///
    /// Wraps the source in a timing loop via `timeit` module.
    /// Returns `None` if `python3` is not available.
    pub fn run_cpython(&self, bench: &Benchmark) -> Option<BenchResult> {
        let python3 = self.python3_bin.as_deref()?;
        let iters = bench.iters.max(1);

        // Build a timing script: run the snippet iters times and print elapsed ns.
        let script = format!(
            r#"
import time as __t
__start = __t.perf_counter_ns()
for __i in range({iters}):
{indented}
__end = __t.perf_counter_ns()
print(__end - __start)
"#,
            iters = iters,
            indented = indent_source(bench.source),
        );

        let output = Command::new(python3).arg("-c").arg(&script).output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let ns: u64 = stdout.trim().parse().ok()?;
        Some(BenchResult {
            total: Duration::from_nanos(ns),
            iters,
        })
    }

    /// Run `bench` under PyPy 7.3 and return timing.
    ///
    /// Wraps the source in a timing loop via `time.perf_counter_ns`.
    /// Returns `None` if `pypy3` is not available.
    pub fn run_pypy(&self, bench: &Benchmark) -> Option<BenchResult> {
        let pypy3 = self.pypy3_bin.as_deref()?;
        let iters = bench.iters.max(1);

        let script = format!(
            r#"
import time as __t
__start = __t.perf_counter_ns()
for __i in range({iters}):
{indented}
__end = __t.perf_counter_ns()
print(__end - __start)
"#,
            iters = iters,
            indented = indent_source(bench.source),
        );

        let output = Command::new(pypy3).arg("-c").arg(&script).output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let ns: u64 = stdout.trim().parse().ok()?;
        Some(BenchResult {
            total: Duration::from_nanos(ns),
            iters,
        })
    }
}

/// Indent every non-empty line of `src` by 4 spaces (for embedding in a for loop).
fn indent_source(src: &str) -> String {
    src.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("    {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Suite ───────────────────────────────────────────────────────────────────

/// A collection of benchmarks to execute.
pub struct BenchSuite {
    pub benchmarks: Vec<Benchmark>,
}

impl BenchSuite {
    /// The built-in benchmark suite covering R1 workloads.
    pub fn builtin() -> Self {
        Self {
            benchmarks: vec![
                // ── Numeric micro-benchmarks ────────────────────────────────
                Benchmark {
                    name: "int_sum_loop",
                    source: r#"
total: int = 0
i: int = 0
while i < 10000:
    total = total + i
    i = i + 1
"#,
                    kind: BenchKind::Numeric,
                    iters: 50,
                },
                Benchmark {
                    name: "int_mul_loop",
                    source: r#"
result: int = 1
i: int = 1
while i <= 20:
    result = result * i
    i = i + 1
"#,
                    kind: BenchKind::Numeric,
                    iters: 100,
                },
                // ── Recursion micro-benchmarks ──────────────────────────────
                Benchmark {
                    name: "fib_recursive",
                    source: r#"
def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

result: int = fib(20)
"#,
                    kind: BenchKind::Recursion,
                    iters: 20,
                },
                Benchmark {
                    name: "factorial_recursive",
                    source: r#"
def fact(n: int) -> int:
    if n <= 1:
        return 1
    return n * fact(n - 1)

result: int = fact(15)
"#,
                    kind: BenchKind::Recursion,
                    iters: 50,
                },
                // ── Range loop (native counter) ────────────────────────────
                Benchmark {
                    name: "range_sum_loop",
                    source: r#"
total: int = 0
for i in range(10000):
    total = total + i
"#,
                    kind: BenchKind::Numeric,
                    iters: 50,
                },
                // ── Generator workload ─────────────────────────────────────
                Benchmark {
                    name: "generator_sum",
                    source: r#"
def gen(n: int):
    i: int = 0
    while i < n:
        yield i
        i = i + 1

total: int = 0
for x in gen(10000):
    total = total + x
"#,
                    kind: BenchKind::Workload,
                    iters: 20,
                },
                // ── Real-world workloads ────────────────────────────────────
                Benchmark {
                    name: "list_sort_builtin",
                    source: r#"
data = [9, 3, 7, 1, 5, 8, 2, 6, 4, 0]
sorted_data = sorted(data)
"#,
                    kind: BenchKind::Workload,
                    iters: 100,
                },
                Benchmark {
                    name: "string_concat",
                    source: r#"
parts = ["hello", " ", "world", "!"]
result = "".join(parts)
"#,
                    kind: BenchKind::Workload,
                    iters: 100,
                },
                // Regression bench for #2128 — tuple-return hot path.
                // Mirrors the colorsys.rgb_to_hls shape isolated in the
                // issue (~150-220x slower than CPython, internal time)
                // due to MakeTuple's transient List allocation being
                // gc_track'd. With the mb_tuple_new_N fast path the
                // intermediate list is gone and primitive tuples skip
                // gc_track entirely.
                Benchmark {
                    name: "tuple_return_scalar",
                    source: r#"
def rgb_to_hls(r: float, g: float, b: float):
    maxc: float = r
    if g > maxc:
        maxc = g
    if b > maxc:
        maxc = b
    minc: float = r
    if g < minc:
        minc = g
    if b < minc:
        minc = b
    s: float = (maxc + minc) * 0.5
    return (maxc, minc, s)

i: int = 0
while i < 1000:
    t = rgb_to_hls(0.5, 0.25, 0.75)
    i = i + 1
"#,
                    kind: BenchKind::Workload,
                    iters: 20,
                },
            ],
        }
    }

    /// Filter benchmarks to only those matching `kind`.
    pub fn filter_kind(&self, kind: BenchKind) -> impl Iterator<Item = &Benchmark> {
        self.benchmarks.iter().filter(move |b| b.kind == kind)
    }
}

// ── Report ──────────────────────────────────────────────────────────────────

/// A row in the comparison report.
pub struct ReportRow {
    pub name: String,
    pub kind: BenchKind,
    pub mamba_ns_mean: Option<u64>,
    pub cpython_ns_mean: Option<u64>,
    /// PyPy 7.3 mean time per iteration (ns). None if PyPy is unavailable or failed.
    pub pypy_ns_mean: Option<u64>,
    pub mamba_error: Option<String>,
}

impl ReportRow {
    /// Speedup factor vs CPython: CPython mean / Mamba mean.
    pub fn speedup(&self) -> Option<f64> {
        let m = self.mamba_ns_mean? as f64;
        let c = self.cpython_ns_mean? as f64;
        if m == 0.0 {
            return None;
        }
        Some(c / m)
    }

    /// Speedup factor vs PyPy: PyPy mean / Mamba mean.
    pub fn speedup_vs_pypy(&self) -> Option<f64> {
        let m = self.mamba_ns_mean? as f64;
        let p = self.pypy_ns_mean? as f64;
        if m == 0.0 {
            return None;
        }
        Some(p / m)
    }
}

/// Print a formatted comparison table to stdout.
pub fn print_report(rows: &[ReportRow]) {
    // Header
    println!();
    println!(
        "{:<30} {:<10} {:>14} {:>14} {:>14} {:>10}",
        "Benchmark", "Kind", "Mamba (ns/op)", "CPython (ns/op)", "PyPy (ns/op)", "vs CPython"
    );
    println!("{}", "-".repeat(98));

    for row in rows {
        let mamba_str = match (row.mamba_ns_mean, &row.mamba_error) {
            (Some(ns), _) => format!("{ns:>14}"),
            (None, Some(e)) => format!("  ERR:{:<9}", truncate(e, 9)),
            (None, None) => "            --".to_string(),
        };
        let cpython_str = match row.cpython_ns_mean {
            Some(ns) => format!("{ns:>14}"),
            None => "            --".to_string(),
        };
        let pypy_str = match row.pypy_ns_mean {
            Some(ns) => format!("{ns:>14}"),
            None => "            --".to_string(),
        };
        let speedup_str = match row.speedup() {
            Some(s) => format!("{s:>9.2}x"),
            None => "        --".to_string(),
        };

        println!(
            "{:<30} {:<10} {} {} {} {}",
            row.name,
            row.kind.label(),
            mamba_str,
            cpython_str,
            pypy_str,
            speedup_str,
        );
    }
    println!("{}", "-".repeat(98));
    println!();
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

// ── Run suite ───────────────────────────────────────────────────────────────

/// Run the full benchmark suite and return report rows.
pub fn run_suite(suite: &BenchSuite, runner: &BenchRunner) -> Vec<ReportRow> {
    suite
        .benchmarks
        .iter()
        .map(|bench| {
            let (mamba_ns_mean, mamba_error) = match runner.run_mamba(bench) {
                Ok(r) => (Some(r.mean().as_nanos() as u64), None),
                Err(e) => (None, Some(e)),
            };
            let cpython_ns_mean = runner
                .run_cpython(bench)
                .map(|r| r.mean().as_nanos() as u64);
            let pypy_ns_mean = runner.run_pypy(bench).map(|r| r.mean().as_nanos() as u64);

            ReportRow {
                name: bench.name.to_string(),
                kind: bench.kind,
                mamba_ns_mean,
                cpython_ns_mean,
                pypy_ns_mean,
                mamba_error,
            }
        })
        .collect()
}

// ── Fixture-based benchmarks ────────────────────────────────────────────────
//
// Each `.py` file in the bench fixtures directory is a self-contained benchmark.
// A matching `.expected` file provides the correct output for validation.
// Both mamba and python3 run the SAME `.py` file as a subprocess — wall-clock
// timing measures actual runtime, not JIT compilation overhead.

use std::path::{Path, PathBuf};

/// A fixture-based benchmark discovered from the filesystem.
pub struct FixtureBench {
    pub name: String,
    pub py_path: PathBuf,
    pub expected: Option<String>,
}

/// Discover `.py` benchmarks in `dir`, loading matching `.expected` files.
pub fn discover_fixtures(dir: &Path) -> Vec<FixtureBench> {
    let mut benches = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return benches;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "py"))
        .collect();
    paths.sort();
    for py_path in paths {
        let name = py_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let expected_path = py_path.with_extension("expected");
        let expected = std::fs::read_to_string(&expected_path).ok();
        benches.push(FixtureBench {
            name,
            py_path,
            expected,
        });
    }
    benches
}

/// Result of running a fixture benchmark with one engine.
pub struct FixtureResult {
    pub elapsed: Duration,
    pub stdout: String,
    pub correct: bool,
}

/// Run a fixture `.py` file with a given engine binary, check output.
fn run_fixture_engine(
    py_path: &Path,
    engine: &str,
    args: &[&str],
    expected: Option<&str>,
) -> Option<FixtureResult> {
    let start = Instant::now();
    let output = Command::new(engine).args(args).arg(py_path).output().ok()?;
    let elapsed = start.elapsed();

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let correct = expected.map_or(true, |exp| stdout.trim() == exp.trim());
    Some(FixtureResult {
        elapsed,
        stdout,
        correct,
    })
}

/// Row in the fixture benchmark report.
pub struct FixtureReportRow {
    pub name: String,
    pub mamba: Option<FixtureResult>,
    pub cpython: Option<FixtureResult>,
}

/// Run all fixture benchmarks with both mamba and CPython.
/// `mamba_bin` is the path to the cclab binary.
pub fn run_fixture_suite(fixtures: &[FixtureBench], mamba_bin: &Path) -> Vec<FixtureReportRow> {
    let has_python = Command::new("python3").arg("--version").output().is_ok();

    fixtures
        .iter()
        .map(|fb| {
            let expected = fb.expected.as_deref();
            let mamba = run_fixture_engine(
                &fb.py_path,
                &mamba_bin.to_string_lossy(),
                &["mamba", "run"],
                expected,
            );
            let cpython = if has_python {
                run_fixture_engine(&fb.py_path, "python3", &[], expected)
            } else {
                None
            };
            FixtureReportRow {
                name: fb.name.clone(),
                mamba,
                cpython,
            }
        })
        .collect()
}

/// Print fixture benchmark report.
pub fn print_fixture_report(rows: &[FixtureReportRow]) {
    println!();
    println!(
        "{:<20} {:>10} {:>10} {:>10} {:>6} {:>6}",
        "Benchmark", "Mamba", "CPython", "Speedup", "M✓", "C✓"
    );
    println!("{}", "-".repeat(68));
    for row in rows {
        let m_str = row
            .mamba
            .as_ref()
            .map(|r| format!("{:.3}s", r.elapsed.as_secs_f64()))
            .unwrap_or_else(|| "ERR".to_string());
        let c_str = row
            .cpython
            .as_ref()
            .map(|r| format!("{:.3}s", r.elapsed.as_secs_f64()))
            .unwrap_or_else(|| "--".to_string());
        let speedup = match (&row.mamba, &row.cpython) {
            (Some(m), Some(c)) if m.elapsed.as_nanos() > 0 => {
                let s = c.elapsed.as_secs_f64() / m.elapsed.as_secs_f64();
                format!("{s:.1}x")
            }
            _ => "--".to_string(),
        };
        let m_ok = row
            .mamba
            .as_ref()
            .map_or("--", |r| if r.correct { "✓" } else { "✗" });
        let c_ok = row
            .cpython
            .as_ref()
            .map_or("--", |r| if r.correct { "✓" } else { "✗" });
        println!(
            "{:<20} {:>10} {:>10} {:>10} {:>6} {:>6}",
            row.name, m_str, c_str, speedup, m_ok, c_ok,
        );
    }
    println!("{}", "-".repeat(68));
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_runner_int_sum() {
        let runner = BenchRunner::mamba_only();
        let bench = Benchmark {
            name: "int_sum",
            source: r#"
total: int = 0
i: int = 0
while i < 100:
    total = total + i
    i = i + 1
"#,
            kind: BenchKind::Numeric,
            iters: 5,
        };
        let result = runner.run_mamba(&bench).expect("bench should succeed");
        assert!(result.iters == 5);
        assert!(result.total > Duration::ZERO);
        // Mean should be well under 10s for a trivial loop
        assert!(
            result.mean() < Duration::from_secs(10),
            "bench too slow: {:?}",
            result.mean()
        );
    }

    #[test]
    fn test_bench_runner_fib() {
        let runner = BenchRunner::mamba_only();
        let bench = Benchmark {
            name: "fib",
            source: r#"
def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

result: int = fib(10)
"#,
            kind: BenchKind::Recursion,
            iters: 3,
        };
        let result = runner.run_mamba(&bench).expect("fib bench should succeed");
        assert_eq!(result.iters, 3);
        assert!(result.total > Duration::ZERO);
    }

    #[test]
    fn test_bench_runner_parse_error_propagates() {
        let runner = BenchRunner::mamba_only();
        let bench = Benchmark {
            name: "bad_syntax",
            source: "def (", // invalid
            kind: BenchKind::Numeric,
            iters: 1,
        };
        let result = runner.run_mamba(&bench);
        assert!(result.is_err(), "should error on parse failure");
    }

    #[test]
    fn test_builtin_suite_has_all_kinds() {
        let suite = BenchSuite::builtin();
        let has_numeric = suite
            .benchmarks
            .iter()
            .any(|b| b.kind == BenchKind::Numeric);
        let has_recursion = suite
            .benchmarks
            .iter()
            .any(|b| b.kind == BenchKind::Recursion);
        let has_workload = suite
            .benchmarks
            .iter()
            .any(|b| b.kind == BenchKind::Workload);
        assert!(has_numeric, "suite must include numeric benchmarks");
        assert!(has_recursion, "suite must include recursion benchmarks");
        assert!(has_workload, "suite must include workload benchmarks");
    }

    #[test]
    fn test_bench_result_mean() {
        let r = BenchResult {
            total: Duration::from_millis(100),
            iters: 4,
        };
        assert_eq!(r.mean(), Duration::from_millis(25));
    }

    #[test]
    fn test_bench_result_mean_zero_iters() {
        let r = BenchResult {
            total: Duration::from_millis(100),
            iters: 0,
        };
        assert_eq!(r.mean(), Duration::ZERO);
    }

    /// #1274 regression: per-iter cost must not grow linearly with cumulative
    /// invocation count. Each main_fn() call previously re-ran
    /// mb_register_builtins → ~130 stdlib container allocations under
    /// gc_disable → no collect → unbounded heap growth. With the idempotency
    /// guard in mb_register_builtins, per-iter time at high N stays close
    /// to per-iter time at low N.
    #[test]
    fn test_heap_workloads_scale_flat() {
        let runner = BenchRunner::mamba_only();
        for source in &[
            r#"
parts = ["hello", " ", "world", "!"]
result = "".join(parts)
"#,
            r#"
data = [9, 3, 7, 1, 5, 8, 2, 6, 4, 0]
sorted_data = sorted(data)
"#,
        ] {
            let warmup = Benchmark {
                name: "warm",
                source,
                kind: BenchKind::Workload,
                iters: 50,
            };
            // Warmup so the first measurement isn't dominated by JIT compile.
            runner.run_mamba(&warmup).expect("warmup");

            let small = Benchmark {
                name: "n10",
                source,
                kind: BenchKind::Workload,
                iters: 10,
            };
            let large = Benchmark {
                name: "n500",
                source,
                kind: BenchKind::Workload,
                iters: 500,
            };
            let s = runner.run_mamba(&small).expect("n=10");
            let l = runner.run_mamba(&large).expect("n=500");
            let s_per = s.mean().as_nanos();
            let l_per = l.mean().as_nanos();
            // Allow 2× slack: scheduler jitter + first-call cache effects, but
            // catch the pre-fix 30× regression. Pre-fix this was >20×. Per
            // #1663's stated acceptance: "per-iter time at N=500 within 2× of
            // N=10". The `s_per + 5_000` floor preserves a noise margin when
            // the small-N measurement is dominated by sampling jitter
            // (sub-microsecond).
            assert!(
                l_per <= s_per.saturating_mul(2).max(s_per + 5_000),
                "scaling regression: N=10 {}ns/iter, N=500 {}ns/iter — \
                 mb_register_builtins idempotency or GC firing may be broken",
                s_per,
                l_per,
            );
        }
    }

    #[test]
    fn test_report_row_speedup() {
        let row = ReportRow {
            name: "test".into(),
            kind: BenchKind::Numeric,
            mamba_ns_mean: Some(100),
            cpython_ns_mean: Some(400),
            pypy_ns_mean: None,
            mamba_error: None,
        };
        let speedup = row.speedup().expect("should have speedup");
        assert!((speedup - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_report_row_speedup_no_cpython() {
        let row = ReportRow {
            name: "test".into(),
            kind: BenchKind::Numeric,
            mamba_ns_mean: Some(100),
            cpython_ns_mean: None,
            pypy_ns_mean: None,
            mamba_error: None,
        };
        assert!(row.speedup().is_none());
    }

    #[test]
    fn test_indent_source() {
        let src = "x: int = 1\ny: int = 2\n";
        let indented = indent_source(src);
        for line in indented.lines() {
            if !line.trim().is_empty() {
                assert!(
                    line.starts_with("    "),
                    "expected 4-space indent, got: {line:?}"
                );
            }
        }
    }

    #[test]
    fn test_run_suite_produces_rows() {
        let suite = BenchSuite {
            benchmarks: vec![Benchmark {
                name: "trivial",
                source: "x: int = 1\n",
                kind: BenchKind::Numeric,
                iters: 2,
            }],
        };
        let runner = BenchRunner::mamba_only();
        let rows = run_suite(&suite, &runner);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "trivial");
    }

    #[test]
    fn test_filter_kind() {
        let suite = BenchSuite::builtin();
        let numerics: Vec<_> = suite.filter_kind(BenchKind::Numeric).collect();
        assert!(!numerics.is_empty());
        assert!(numerics.iter().all(|b| b.kind == BenchKind::Numeric));
    }

    #[test]
    fn test_print_report_does_not_panic() {
        // Smoke test: printing should not panic with partial data.
        let rows = vec![
            ReportRow {
                name: "smoke".into(),
                kind: BenchKind::Numeric,
                mamba_ns_mean: Some(1000),
                cpython_ns_mean: Some(5000),
                pypy_ns_mean: Some(2000),
                mamba_error: None,
            },
            ReportRow {
                name: "err_bench".into(),
                kind: BenchKind::Recursion,
                mamba_ns_mean: None,
                cpython_ns_mean: None,
                pypy_ns_mean: None,
                mamba_error: Some("parse failed".into()),
            },
        ];
        print_report(&rows); // must not panic
    }
}
