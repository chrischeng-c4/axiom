// Performance benchmark harness vs uv (Tick 29).
//
// uv's own benchmark suite publishes numbers like
//   `uv pip install <set>` warm cache: 10× pip
//   cold cache:                          8× pip
//   `uv lock` warm:                      80× pip-tools
// We need a comparable harness so mamba's pkg-mgmt features can be
// regressed against uv on the same fixture.
//
// This module is the *data layer + thin runner* of that harness. It:
//   * spawns an external command N times,
//   * records wall-clock time per run,
//   * computes summary stats (mean / median / min / max / p95),
//   * pairs two `BenchSummary`s into a `Comparison` and emits a verdict
//     (FasterThan, ParityWith, SlowerThan) bracketed by a user-supplied
//     parity threshold.
//
// What it deliberately is NOT:
//   * a perf regression GUARD wired into CI — that's a future tick,
//     gated by whether we want to fail builds on perf drift.
//   * a JIT-warming harness — wall-clock comparisons against uv are
//     subprocess-spawn comparisons. We capture exit_code + stderr so
//     callers can spot tool failure (it would otherwise show as a fast
//     time-to-exit and silently win the comparison).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One bench definition: program + args + working directory. Cloned per
/// run so warmup vs measure share identical invocation.
#[derive(Debug, Clone)]
pub struct BenchCommand {
    /// Binary to invoke (absolute path preferred).
    pub program: PathBuf,
    /// Arguments to pass.
    pub args: Vec<String>,
    /// Optional cwd. None = inherit current process cwd.
    pub cwd: Option<PathBuf>,
    /// Free-form label. Shown in summaries to disambiguate which
    /// subcommand was measured ("mamba sync (warm)").
    pub label: String,
}

/// Knobs for one bench run.
#[derive(Debug, Clone)]
pub struct BenchOptions {
    /// How many warmup runs to discard (lets disk caches settle).
    pub warmup: u32,
    /// How many measured runs to keep.
    pub iterations: u32,
    /// Per-run wall-clock cap. Runs that exceed are killed and recorded
    /// as failed; the summary keeps them in `failures`.
    pub timeout: Duration,
}

impl Default for BenchOptions {
    fn default() -> Self {
        Self {
            warmup: 1,
            iterations: 5,
            timeout: Duration::from_secs(300),
        }
    }
}

/// One measured run.
#[derive(Debug, Clone)]
pub struct BenchSample {
    pub wall: Duration,
    pub exit_code: Option<i32>,
    pub stderr_tail: String,
}

impl BenchSample {
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Aggregated stats from a bench run.
#[derive(Debug, Clone)]
pub struct BenchSummary {
    pub label: String,
    pub samples: Vec<BenchSample>,
    pub failures: usize,
}

impl BenchSummary {
    /// Only successful samples are used for stats.
    fn successful_walls(&self) -> Vec<Duration> {
        self.samples
            .iter()
            .filter(|s| s.succeeded())
            .map(|s| s.wall)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.successful_walls().len()
    }

    pub fn min(&self) -> Option<Duration> {
        self.successful_walls().into_iter().min()
    }

    pub fn max(&self) -> Option<Duration> {
        self.successful_walls().into_iter().max()
    }

    pub fn mean(&self) -> Option<Duration> {
        let walls = self.successful_walls();
        if walls.is_empty() {
            return None;
        }
        let total: u128 = walls.iter().map(|d| d.as_nanos()).sum();
        let avg = total / walls.len() as u128;
        Some(Duration::from_nanos(avg as u64))
    }

    pub fn median(&self) -> Option<Duration> {
        let mut walls = self.successful_walls();
        if walls.is_empty() {
            return None;
        }
        walls.sort();
        let mid = walls.len() / 2;
        if walls.len() % 2 == 1 {
            Some(walls[mid])
        } else {
            let a = walls[mid - 1].as_nanos();
            let b = walls[mid].as_nanos();
            Some(Duration::from_nanos(((a + b) / 2) as u64))
        }
    }

    pub fn p95(&self) -> Option<Duration> {
        let mut walls = self.successful_walls();
        if walls.is_empty() {
            return None;
        }
        walls.sort();
        // Nearest-rank: index = ceil(0.95 * N) - 1, clamped.
        let n = walls.len();
        let rank = ((0.95 * n as f64).ceil() as usize).saturating_sub(1);
        Some(walls[rank.min(n - 1)])
    }
}

/// Side-by-side ratio of mamba vs uv on the same fixture.
#[derive(Debug, Clone)]
pub struct Comparison {
    pub mamba: BenchSummary,
    pub baseline: BenchSummary,
    /// mamba_median / baseline_median. < 1.0 means mamba is faster.
    pub ratio: f64,
    pub verdict: ComparisonVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonVerdict {
    /// mamba faster than baseline by more than the parity band.
    FasterThan,
    /// Within ±parity_band of baseline.
    ParityWith,
    /// mamba slower than baseline by more than the parity band.
    SlowerThan,
    /// One or both summaries had no successful samples; ratio is NaN
    /// and no meaningful verdict can be issued.
    Inconclusive,
}

/// Compare two summaries.
///
/// `parity_band` is fractional, e.g. 0.05 = ±5%. If `|ratio - 1| <=
/// parity_band`, the verdict is `ParityWith`.
pub fn compare(mamba: BenchSummary, baseline: BenchSummary, parity_band: f64) -> Comparison {
    let (m_median, b_median) = (mamba.median(), baseline.median());
    let (Some(m), Some(b)) = (m_median, b_median) else {
        return Comparison {
            mamba,
            baseline,
            ratio: f64::NAN,
            verdict: ComparisonVerdict::Inconclusive,
        };
    };
    if b.is_zero() {
        return Comparison {
            mamba,
            baseline,
            ratio: f64::NAN,
            verdict: ComparisonVerdict::Inconclusive,
        };
    }
    let ratio = m.as_secs_f64() / b.as_secs_f64();
    let verdict = if (ratio - 1.0).abs() <= parity_band {
        ComparisonVerdict::ParityWith
    } else if ratio < 1.0 {
        ComparisonVerdict::FasterThan
    } else {
        ComparisonVerdict::SlowerThan
    };
    Comparison {
        mamba,
        baseline,
        ratio,
        verdict,
    }
}

/// Run one `BenchCommand` per `BenchOptions` and return its `BenchSummary`.
///
/// Warmup runs are executed and discarded; only the `iterations` measured
/// runs are recorded. A run that times out or returns a non-zero exit
/// code is recorded as a `BenchSample` with `succeeded() == false` and
/// counts toward `failures`, but does not abort the bench — callers want
/// to *see* the failure pattern, not be early-returned past it.
pub fn run_bench(cmd: &BenchCommand, opts: &BenchOptions) -> Result<BenchSummary, IndexError> {
    for _ in 0..opts.warmup {
        let _ = run_once(cmd, opts.timeout)?;
    }
    let mut samples = Vec::with_capacity(opts.iterations as usize);
    let mut failures = 0usize;
    for _ in 0..opts.iterations {
        let s = run_once(cmd, opts.timeout)?;
        if !s.succeeded() {
            failures += 1;
        }
        samples.push(s);
    }
    Ok(BenchSummary {
        label: cmd.label.clone(),
        samples,
        failures,
    })
}

fn run_once(cmd: &BenchCommand, timeout: Duration) -> Result<BenchSample, IndexError> {
    let mut command = Command::new(&cmd.program);
    command.args(&cmd.args);
    if let Some(cwd) = &cmd.cwd {
        command.current_dir(cwd);
    }
    command.stdout(Stdio::null());
    command.stderr(Stdio::piped());

    let started = Instant::now();
    let mut child = command.spawn().map_err(|err| IndexError::NetworkError {
        url: cmd.program.display().to_string(),
        detail: format!("spawning bench command {:?}: {err}", cmd.label),
    })?;

    // Lightweight wait-with-timeout: poll try_wait at coarse intervals.
    // The Python ecosystem's bench commands typically run tens of ms to
    // tens of seconds, so 10 ms polling is acceptable overhead.
    let deadline = started + timeout;
    let exit_status;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                exit_status = Some(status);
                break;
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    exit_status = None;
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(err) => {
                return Err(IndexError::NetworkError {
                    url: cmd.program.display().to_string(),
                    detail: format!("try_wait on bench {:?}: {err}", cmd.label),
                });
            }
        }
    }
    let wall = started.elapsed();

    // Drain stderr. If we killed the child for timeout, we still want
    // the head of its stderr buffer (often shows where it hung).
    let stderr_tail = read_stderr_tail(child);

    let exit_code = exit_status.and_then(|s| s.code());

    Ok(BenchSample {
        wall,
        exit_code,
        stderr_tail,
    })
}

fn read_stderr_tail(child: std::process::Child) -> String {
    use std::io::Read;
    let mut output = String::new();
    if let Some(mut err) = child.stderr {
        let _ = err.read_to_string(&mut output);
    }
    // Cap at 2 KiB so a misbehaving tool can't blow up the summary.
    if output.len() > 2048 {
        let cutoff = output.len() - 2048;
        output = format!("...{}", &output[cutoff..]);
    }
    output
}

/// Render a comparison as a plain-text table line.
///
/// Format: `<label> | mamba <median>ms ± <p95>ms | uv <median>ms ± <p95>ms | <ratio>× | <verdict>`
/// where the verdict is "faster"/"parity"/"slower"/"n/a".
pub fn render_text(c: &Comparison) -> String {
    let label = if c.mamba.label == c.baseline.label {
        c.mamba.label.clone()
    } else {
        format!("{} vs {}", c.mamba.label, c.baseline.label)
    };
    let fmt = |d: Option<Duration>| match d {
        Some(d) => format!("{}ms", d.as_millis()),
        None => "—".into(),
    };
    let ratio = if c.ratio.is_nan() {
        "n/a".into()
    } else {
        format!("{:.2}×", c.ratio)
    };
    let verdict = match c.verdict {
        ComparisonVerdict::FasterThan => "faster",
        ComparisonVerdict::ParityWith => "parity",
        ComparisonVerdict::SlowerThan => "slower",
        ComparisonVerdict::Inconclusive => "n/a",
    };
    format!(
        "{} | mamba {} (p95 {}) | base {} (p95 {}) | {} | {}",
        label,
        fmt(c.mamba.median()),
        fmt(c.mamba.p95()),
        fmt(c.baseline.median()),
        fmt(c.baseline.p95()),
        ratio,
        verdict,
    )
}

/// First entry on PATH named `program` that is a regular file.
/// Useful for tests that opportunistically use the system `python3`.
pub fn locate_on_path(program: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Diagnostic helper: classify a run for one-line logging.
pub fn classify_sample(s: &BenchSample) -> &'static str {
    match s.exit_code {
        Some(0) => "ok",
        Some(_) => "exit-nonzero",
        None => "killed-timeout",
    }
}

/// Hint helper: was the working directory present at bench time?
/// Surfaces "missing fixture" before the user spends 5 minutes wondering
/// why every iteration exits in 3ms.
pub fn fixture_ready(cwd: &Path) -> bool {
    cwd.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn mk_sample(ms: u64, ok: bool) -> BenchSample {
        BenchSample {
            wall: Duration::from_millis(ms),
            exit_code: if ok { Some(0) } else { Some(1) },
            stderr_tail: String::new(),
        }
    }

    fn mk_summary(label: &str, walls: &[(u64, bool)]) -> BenchSummary {
        let samples: Vec<BenchSample> = walls.iter().map(|(ms, ok)| mk_sample(*ms, *ok)).collect();
        let failures = samples.iter().filter(|s| !s.succeeded()).count();
        BenchSummary {
            label: label.into(),
            samples,
            failures,
        }
    }

    #[test]
    fn summary_stats_basic() {
        let s = mk_summary(
            "x",
            &[
                (100, true),
                (200, true),
                (300, true),
                (400, true),
                (500, true),
            ],
        );
        assert_eq!(s.count(), 5);
        assert_eq!(s.min(), Some(Duration::from_millis(100)));
        assert_eq!(s.max(), Some(Duration::from_millis(500)));
        assert_eq!(s.mean(), Some(Duration::from_millis(300)));
        assert_eq!(s.median(), Some(Duration::from_millis(300)));
        // p95 over 5 samples: ceil(4.75)-1 = 4 → 500ms.
        assert_eq!(s.p95(), Some(Duration::from_millis(500)));
    }

    #[test]
    fn summary_median_even_count_averages() {
        let s = mk_summary("x", &[(100, true), (200, true), (300, true), (400, true)]);
        // Median is (200 + 300) / 2 = 250ms.
        assert_eq!(s.median(), Some(Duration::from_millis(250)));
    }

    #[test]
    fn summary_ignores_failed_runs() {
        let s = mk_summary(
            "x",
            &[(100, true), (10_000, false), (200, true), (300, true)],
        );
        assert_eq!(s.count(), 3, "failed run must not enter stats");
        assert_eq!(s.failures, 1);
        assert_eq!(s.median(), Some(Duration::from_millis(200)));
    }

    #[test]
    fn summary_empty_yields_none() {
        let s = mk_summary("x", &[]);
        assert_eq!(s.count(), 0);
        assert!(s.median().is_none());
        assert!(s.p95().is_none());
    }

    #[test]
    fn compare_parity_within_band() {
        let a = mk_summary("mamba", &[(100, true)]);
        let b = mk_summary("uv", &[(103, true)]);
        let c = compare(a, b, 0.05);
        assert_eq!(c.verdict, ComparisonVerdict::ParityWith);
        assert!((c.ratio - (100.0 / 103.0)).abs() < 1e-9);
    }

    #[test]
    fn compare_faster_when_below_band() {
        let a = mk_summary("mamba", &[(50, true)]);
        let b = mk_summary("uv", &[(100, true)]);
        let c = compare(a, b, 0.05);
        assert_eq!(c.verdict, ComparisonVerdict::FasterThan);
        assert!(c.ratio < 1.0);
    }

    #[test]
    fn compare_slower_when_above_band() {
        let a = mk_summary("mamba", &[(200, true)]);
        let b = mk_summary("uv", &[(100, true)]);
        let c = compare(a, b, 0.05);
        assert_eq!(c.verdict, ComparisonVerdict::SlowerThan);
        assert!(c.ratio > 1.0);
    }

    #[test]
    fn compare_inconclusive_when_one_side_empty() {
        let a = mk_summary("mamba", &[]);
        let b = mk_summary("uv", &[(100, true)]);
        let c = compare(a, b, 0.05);
        assert_eq!(c.verdict, ComparisonVerdict::Inconclusive);
        assert!(c.ratio.is_nan());
    }

    #[test]
    fn compare_inconclusive_when_baseline_zero() {
        // Edge case: baseline ran but reported 0ms wall.
        let a = mk_summary("mamba", &[(50, true)]);
        let b = mk_summary("uv", &[(0, true)]);
        let c = compare(a, b, 0.05);
        assert_eq!(c.verdict, ComparisonVerdict::Inconclusive);
    }

    #[test]
    fn render_text_includes_label_ratio_and_verdict() {
        let a = mk_summary("install", &[(100, true)]);
        let b = mk_summary("install", &[(200, true)]);
        let c = compare(a, b, 0.05);
        let line = render_text(&c);
        assert!(line.contains("install"));
        assert!(line.contains("0.50×"));
        assert!(line.contains("faster"));
    }

    #[test]
    fn render_text_handles_inconclusive_gracefully() {
        let a = mk_summary("mamba", &[]);
        let b = mk_summary("uv", &[(50, true)]);
        let c = compare(a, b, 0.05);
        let line = render_text(&c);
        assert!(line.contains("n/a"));
    }

    #[test]
    fn classify_sample_buckets_correctly() {
        assert_eq!(classify_sample(&mk_sample(1, true)), "ok");
        assert_eq!(classify_sample(&mk_sample(1, false)), "exit-nonzero");
        let killed = BenchSample {
            wall: Duration::from_millis(5),
            exit_code: None,
            stderr_tail: String::new(),
        };
        assert_eq!(classify_sample(&killed), "killed-timeout");
    }

    #[test]
    fn fixture_ready_detects_missing_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        assert!(fixture_ready(dir.path()));
        let bogus = dir.path().join("nope");
        assert!(!fixture_ready(&bogus));
    }

    #[test]
    fn run_bench_real_subprocess_smoke() {
        // Use whichever `python3`-ish binary is on PATH for a tiny
        // command. Skip cleanly if there's nothing usable.
        let Some(python) = locate_on_path("python3").or_else(|| locate_on_path("python")) else {
            eprintln!("(no python on PATH — skipping run_bench_real_subprocess_smoke)");
            return;
        };
        let cmd = BenchCommand {
            program: python,
            args: vec!["-c".into(), "pass".into()],
            cwd: None,
            label: "py-noop".into(),
        };
        let opts = BenchOptions {
            warmup: 0,
            iterations: 2,
            timeout: Duration::from_secs(30),
        };
        let summary = run_bench(&cmd, &opts).expect("bench should succeed");
        assert_eq!(summary.failures, 0);
        assert_eq!(summary.count(), 2);
        assert!(summary.median().is_some());
    }

    #[test]
    fn run_bench_records_failure_for_bad_command() {
        let Some(python) = locate_on_path("python3").or_else(|| locate_on_path("python")) else {
            eprintln!("(no python on PATH — skipping run_bench_records_failure_for_bad_command)");
            return;
        };
        let cmd = BenchCommand {
            program: python,
            args: vec!["-c".into(), "import sys; sys.exit(3)".into()],
            cwd: None,
            label: "py-exit-3".into(),
        };
        let opts = BenchOptions {
            warmup: 0,
            iterations: 2,
            timeout: Duration::from_secs(30),
        };
        let summary = run_bench(&cmd, &opts).expect("spawn should succeed");
        assert_eq!(summary.failures, 2);
        assert_eq!(
            summary.count(),
            0,
            "exit-3 runs must not contribute to stats"
        );
    }

    #[test]
    fn run_bench_spawn_failure_propagates_index_error() {
        let cmd = BenchCommand {
            program: PathBuf::from("/definitely/not/a/program"),
            args: vec![],
            cwd: None,
            label: "missing".into(),
        };
        let opts = BenchOptions {
            warmup: 0,
            iterations: 1,
            timeout: Duration::from_secs(5),
        };
        let err = run_bench(&cmd, &opts).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("spawning bench command"), "got: {msg}");
    }
}
