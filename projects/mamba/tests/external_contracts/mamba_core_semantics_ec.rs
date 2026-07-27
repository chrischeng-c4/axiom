// SPEC-MANAGED: executable evidence for the Mamba Tier 1 to_thread/gather ECs.
// @ec mamba-t1-to-thread-gather-results
// @ec mamba-t1-to-thread-gather-stability
// @ec mamba-t1-to-thread-gather-efficiency

#![cfg(unix)]

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const MIB: u64 = 1024 * 1024;
const RESULTS_TIMEOUT: Duration = Duration::from_secs(30);
const STABILITY_TIMEOUT: Duration = Duration::from_secs(90);
const EFFICIENCY_TIMEOUT: Duration = Duration::from_secs(45);

struct MeasuredRun {
    status: ExitStatus,
    stdout: String,
    stderr: String,
    wall: Duration,
    cpu_seconds: f64,
    peak_rss_bytes: u64,
    timed_out: bool,
}

fn mamba_bin() -> OsString {
    OsString::from(env!("CARGO_BIN_EXE_mamba"))
}

fn write_script(prefix: &str, source: &str) -> tempfile::NamedTempFile {
    let mut script = tempfile::Builder::new()
        .prefix(prefix)
        .suffix(".py")
        .tempfile()
        .expect("create EC Python program");
    script
        .write_all(source.as_bytes())
        .expect("write EC Python program");
    script.flush().expect("flush EC Python program");
    script
}

fn run_mamba_script(source: &str, timeout: Duration) -> MeasuredRun {
    let script = write_script("mamba-t1-ec-", source);
    let args = vec![
        OsString::from("run"),
        script.path().as_os_str().to_os_string(),
    ];
    run_measured(mamba_bin().as_os_str(), &args, timeout)
}

fn run_cpython_script(source: &str, timeout: Duration) -> MeasuredRun {
    let script = write_script("mamba-t1-cpython-control-", source);
    let args = vec![script.path().as_os_str().to_os_string()];
    run_measured(OsStr::new("python3.12"), &args, timeout)
}

fn run_measured(program: &OsStr, args: &[OsString], timeout: Duration) -> MeasuredRun {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("spawn {:?}: {error}", program));
    let pid = child.id() as libc::pid_t;

    let mut stdout_pipe = child.stdout.take().expect("capture child stdout");
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout_pipe
            .read_to_end(&mut bytes)
            .expect("read child stdout");
        bytes
    });
    let mut stderr_pipe = child.stderr.take().expect("capture child stderr");
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr_pipe
            .read_to_end(&mut bytes)
            .expect("read child stderr");
        bytes
    });

    let completion = Arc::new((Mutex::new(false), Condvar::new()));
    let timed_out = Arc::new(AtomicBool::new(false));
    let watchdog_completion = Arc::clone(&completion);
    let watchdog_timed_out = Arc::clone(&timed_out);
    let watchdog = thread::spawn(move || {
        let (lock, wake) = &*watchdog_completion;
        let done = lock.lock().expect("watchdog completion lock");
        let (done, wait) = wake
            .wait_timeout_while(done, timeout, |done| !*done)
            .expect("watchdog wait");
        if !*done && wait.timed_out() {
            watchdog_timed_out.store(true, Ordering::SeqCst);
            unsafe {
                libc::kill(pid, libc::SIGKILL);
            }
        }
    });

    let started = Instant::now();
    let mut raw_status = 0;
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    let waited = unsafe { libc::wait4(pid, &mut raw_status, 0, &mut usage) };
    let wall = started.elapsed();
    {
        let (lock, wake) = &*completion;
        *lock.lock().expect("completion lock") = true;
        wake.notify_all();
    }
    watchdog.join().expect("join process watchdog");
    assert_eq!(waited, pid, "wait4 failed for child {pid}");
    drop(child);

    let stdout =
        String::from_utf8_lossy(&stdout_reader.join().expect("join stdout reader")).into_owned();
    let stderr =
        String::from_utf8_lossy(&stderr_reader.join().expect("join stderr reader")).into_owned();
    MeasuredRun {
        status: ExitStatus::from_raw(raw_status),
        stdout,
        stderr,
        wall,
        cpu_seconds: timeval_seconds(usage.ru_utime) + timeval_seconds(usage.ru_stime),
        peak_rss_bytes: normalize_max_rss(usage.ru_maxrss),
        timed_out: timed_out.load(Ordering::SeqCst),
    }
}

fn timeval_seconds(value: libc::timeval) -> f64 {
    value.tv_sec as f64 + value.tv_usec as f64 / 1_000_000.0
}

#[cfg(target_os = "macos")]
fn normalize_max_rss(value: libc::c_long) -> u64 {
    value.max(0) as u64
}

#[cfg(not(target_os = "macos"))]
fn normalize_max_rss(value: libc::c_long) -> u64 {
    (value.max(0) as u64).saturating_mul(1024)
}

fn assert_success(label: &str, run: &MeasuredRun) {
    assert!(
        !run.timed_out,
        "{label} timed out after {:.3}s\nstdout:\n{}\nstderr:\n{}",
        run.wall.as_secs_f64(),
        run.stdout,
        run.stderr
    );
    assert!(
        run.status.success(),
        "{label} failed ({:?})\nstdout:\n{}\nstderr:\n{}",
        run.status,
        run.stdout,
        run.stderr
    );
}

fn stable_digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="#1841" reason="unit-test section in mamba_core_semantics_ec.rs is hand-written pending codegen support">
#[test]
fn to_thread_gather_results() {
    let source = r#"
import asyncio

def cpu_work(seed: int, work: int) -> int:
    return sum((i ^ seed) * 3 for i in range(work))

async def run_round(rep: int) -> list[int]:
    first_work = 80000 if rep % 2 == 0 else 25000
    second_work = 25000 if rep % 2 == 0 else 80000
    gathered = await asyncio.gather(
        asyncio.to_thread(cpu_work, 101, first_work),
        asyncio.to_thread(cpu_work, 211, second_work),
    )
    expected = [
        cpu_work(101, first_work),
        cpu_work(211, second_work),
    ]
    assert gathered == expected
    assert len(gathered) == 2
    assert gathered[0] is not None
    assert gathered[1] is not None
    assert gathered[0] != gathered[1]
    return list(gathered)

async def main() -> None:
    for rep in range(5):
        gathered = await run_round(rep)
        print("ROUND_OK", rep, gathered[0], gathered[1])

asyncio.run(main())
"#;

    let mamba = run_mamba_script(source, RESULTS_TIMEOUT);
    assert_success("Mamba public asyncio gather program", &mamba);
    let cpython = run_cpython_script(source, RESULTS_TIMEOUT);
    assert_success("CPython 3.12 control program", &cpython);

    assert_eq!(
        mamba.stdout, cpython.stdout,
        "Mamba gather results diverged from CPython control\nMamba stderr:\n{}\nCPython stderr:\n{}",
        mamba.stderr, cpython.stderr
    );
    let lines: Vec<_> = mamba
        .stdout
        .lines()
        .filter(|line| line.starts_with("ROUND_OK "))
        .collect();
    assert_eq!(lines.len(), 5, "expected five exact gather repetitions");
    println!(
        "MAMBA-T1-FT-GATHER-RESULTS rounds=5 digest={:016x}",
        stable_digest(mamba.stdout.as_bytes())
    );
}
// </HANDWRITE>

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StabilityPhase {
    Waiting,
    Baseline,
    WindowOne,
    WindowTwo,
    Quiescent,
}

#[derive(Default)]
struct StabilitySamples {
    baseline_threads: Option<u64>,
    post_threads: Option<u64>,
    window_one_peak_rss: u64,
    window_two_peak_rss: u64,
}

#[cfg(target_os = "macos")]
fn sample_process(pid: u32) -> Result<Option<(u64, u64)>, String> {
    let rss_output = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .map_err(|error| format!("spawn ps RSS sampler for PID {pid}: {error}"))?;
    if !rss_output.status.success() {
        let stderr = String::from_utf8_lossy(&rss_output.stderr);
        return if stderr.trim().is_empty() || stderr.contains("No such process") {
            Ok(None)
        } else {
            Err(format!("ps RSS sampler failed for PID {pid}: {stderr}"))
        };
    }
    let rss_text = String::from_utf8_lossy(&rss_output.stdout);
    if rss_text.trim().is_empty() {
        return Ok(None);
    }
    let rss_kib = rss_text
        .trim()
        .parse::<u64>()
        .map_err(|error| format!("parse ps RSS for PID {pid}: {error}"))?;

    let thread_output = Command::new("ps")
        .args(["-M", "-p", &pid.to_string()])
        .output()
        .map_err(|error| format!("spawn ps thread sampler for PID {pid}: {error}"))?;
    if !thread_output.status.success() {
        let stderr = String::from_utf8_lossy(&thread_output.stderr);
        return if stderr.trim().is_empty() || stderr.contains("No such process") {
            Ok(None)
        } else {
            Err(format!("ps thread sampler failed for PID {pid}: {stderr}"))
        };
    }
    let thread_count = String::from_utf8_lossy(&thread_output.stdout)
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .count() as u64;
    if thread_count == 0 {
        return Ok(None);
    }
    Ok(Some((thread_count, rss_kib.saturating_mul(1024))))
}

#[cfg(not(target_os = "macos"))]
fn sample_process(pid: u32) -> Result<Option<(u64, u64)>, String> {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-o", "nlwp=", "-p", &pid.to_string()])
        .output()
        .map_err(|error| format!("spawn ps for PID {pid}: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return if stderr.trim().is_empty() || stderr.contains("No such process") {
            Ok(None)
        } else {
            Err(format!("ps failed for PID {pid}: {stderr}"))
        };
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let values: Vec<u64> = text
        .split_whitespace()
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|error| format!("parse ps value {value:?}: {error}"))
        })
        .collect::<Result<_, _>>()?;
    if values.len() != 2 {
        return if text.trim().is_empty() {
            Ok(None)
        } else {
            Err(format!(
                "expected RSS and thread count from ps, got {text:?}"
            ))
        };
    }
    Ok(Some((values[1], values[0].saturating_mul(1024))))
}

fn observe_stability_line(line: &str, rounds: &mut BTreeSet<u32>) {
    if let Some(value) = line.strip_prefix("ROUND_OK ") {
        let round = value
            .parse::<u32>()
            .unwrap_or_else(|error| panic!("invalid round evidence {line:?}: {error}"));
        assert!(round < 100, "out-of-range round evidence: {round}");
        assert!(rounds.insert(round), "duplicate round evidence: {round}");
    }
}

fn stability_phase(marker: &str) -> StabilityPhase {
    match marker.trim() {
        "BASELINE_READY" => StabilityPhase::Baseline,
        "WINDOW1_BEGIN" => StabilityPhase::WindowOne,
        "WINDOW2_BEGIN" => StabilityPhase::WindowTwo,
        "QUIESCENT_READY" => StabilityPhase::Quiescent,
        _ => StabilityPhase::Waiting,
    }
}

fn write_stability_ack(stdin: &mut impl Write, phase: &str) {
    writeln!(stdin, "{phase}")
        .unwrap_or_else(|error| panic!("write stability stdin ack {phase}: {error}"));
    stdin
        .flush()
        .unwrap_or_else(|error| panic!("flush stability stdin ack {phase}: {error}"));
}

fn record_stability_sample(
    phase: StabilityPhase,
    threads: u64,
    rss: u64,
    samples: &mut StabilitySamples,
) {
    match phase {
        StabilityPhase::Baseline => {
            samples.baseline_threads = Some(
                samples
                    .baseline_threads
                    .map_or(threads, |current| current.max(threads)),
            );
        }
        StabilityPhase::WindowOne => {
            samples.window_one_peak_rss = samples.window_one_peak_rss.max(rss);
        }
        StabilityPhase::WindowTwo => {
            samples.window_two_peak_rss = samples.window_two_peak_rss.max(rss);
        }
        StabilityPhase::Quiescent => {
            samples.post_threads = Some(
                samples
                    .post_threads
                    .map_or(threads, |current| current.max(threads)),
            );
        }
        StabilityPhase::Waiting => {}
    }
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="#1979" reason="unit-test section in mamba_core_semantics_ec.rs is hand-written pending codegen support">
#[test]
fn to_thread_gather_stability() {
    let marker_dir = tempfile::tempdir().expect("create stability marker directory");
    let marker_path = marker_dir.path().join("phase");
    let marker_literal = marker_path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let source = format!(
        r#"
import asyncio
import sys

def mark(phase: str) -> None:
    marker = open("{marker_literal}", "w")
    marker.write(phase)
    marker.close()

def mark_and_wait(phase: str) -> None:
    mark(phase)
    ack = sys.stdin.readline()
    assert ack.strip() == phase

def cpu_work(seed: int, work: int) -> int:
    return sum((i ^ seed) * 3 for i in range(work))

async def run_round(rep: int) -> None:
    w0 = 3000 + ((rep + 0) % 8) * 500
    w1 = 3000 + ((rep + 1) % 8) * 500
    w2 = 3000 + ((rep + 2) % 8) * 500
    w3 = 3000 + ((rep + 3) % 8) * 500
    w4 = 3000 + ((rep + 4) % 8) * 500
    w5 = 3000 + ((rep + 5) % 8) * 500
    w6 = 3000 + ((rep + 6) % 8) * 500
    w7 = 3000 + ((rep + 7) % 8) * 500
    gathered = await asyncio.gather(
        asyncio.to_thread(cpu_work, 11, w0),
        asyncio.to_thread(cpu_work, 23, w1),
        asyncio.to_thread(cpu_work, 37, w2),
        asyncio.to_thread(cpu_work, 53, w3),
        asyncio.to_thread(cpu_work, 71, w4),
        asyncio.to_thread(cpu_work, 89, w5),
        asyncio.to_thread(cpu_work, 107, w6),
        asyncio.to_thread(cpu_work, 131, w7),
    )
    expected = [
        cpu_work(11, w0),
        cpu_work(23, w1),
        cpu_work(37, w2),
        cpu_work(53, w3),
        cpu_work(71, w4),
        cpu_work(89, w5),
        cpu_work(107, w6),
        cpu_work(131, w7),
    ]
    assert gathered == expected
    assert len(gathered) == 8
    assert len(set(gathered)) == 8
    for item in gathered:
        assert item is not None
    print("ROUND_OK", rep)

async def main() -> None:
    mark_and_wait("BASELINE_READY")
    print("BASELINE_READY")
    mark_and_wait("WINDOW1_BEGIN")
    print("WINDOW1_BEGIN")
    for rep in range(50):
        await run_round(rep)
    mark("WINDOW1_END")
    print("WINDOW1_END")
    mark_and_wait("WINDOW2_BEGIN")
    print("WINDOW2_BEGIN")
    for rep in range(50, 100):
        await run_round(rep)
    mark("WINDOW2_END")
    print("WINDOW2_END")
    mark_and_wait("SOAK_DONE")
    print("SOAK_DONE")
    mark_and_wait("QUIESCENT_READY")
    print("QUIESCENT")

asyncio.run(main())
"#
    );

    let script = write_script("mamba-t1-stability-", &source);
    let mut child = Command::new(mamba_bin())
        .args([OsStr::new("run"), script.path().as_os_str()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn Mamba stability program");
    let pid = child.id();
    let mut child_stdin = child.stdin.take().expect("capture stability stdin");

    let stdout = child.stdout.take().expect("capture stability stdout");
    let (line_tx, line_rx) = mpsc::channel();
    let stdout_reader = thread::spawn(move || {
        let mut lines = Vec::new();
        for line in BufReader::new(stdout).lines() {
            let line = line.expect("read stability stdout line");
            line_tx.send(line.clone()).ok();
            lines.push(line);
        }
        lines.join("\n")
    });
    let mut stderr = child.stderr.take().expect("capture stability stderr");
    let stderr_reader = thread::spawn(move || {
        let mut text = String::new();
        stderr
            .read_to_string(&mut text)
            .expect("read stability stderr");
        text
    });

    let deadline = Instant::now() + STABILITY_TIMEOUT;
    let mut phase = StabilityPhase::Waiting;
    let mut rounds = BTreeSet::new();
    let mut samples = StabilitySamples::default();
    let mut last_marker = String::new();
    let mut last_sample = None;
    let mut baseline_acked = false;
    let mut window_one_acked = false;
    let mut window_two_acked = false;
    let mut soak_done_at = None;
    let mut soak_done_acked = false;
    let mut quiescent_acked = false;
    let status = loop {
        while let Ok(line) = line_rx.try_recv() {
            observe_stability_line(&line, &mut rounds);
        }
        if let Ok(marker) = std::fs::read_to_string(&marker_path) {
            last_marker = marker.trim().to_owned();
            phase = stability_phase(&marker);
            if last_marker == "SOAK_DONE" {
                let observed_at = *soak_done_at.get_or_insert_with(Instant::now);
                if !soak_done_acked && observed_at.elapsed() >= Duration::from_millis(250) {
                    write_stability_ack(&mut child_stdin, "SOAK_DONE");
                    soak_done_acked = true;
                }
            }
        }
        if phase != StabilityPhase::Waiting {
            let sample = sample_process(pid)
                .unwrap_or_else(|error| panic!("OS-visible stability sampling failed: {error}"));
            if let Some((threads, rss)) = sample {
                record_stability_sample(phase, threads, rss, &mut samples);
                last_sample = Some((phase, threads, rss));
                match phase {
                    StabilityPhase::Baseline if !baseline_acked => {
                        write_stability_ack(&mut child_stdin, "BASELINE_READY");
                        baseline_acked = true;
                    }
                    StabilityPhase::WindowOne if !window_one_acked => {
                        write_stability_ack(&mut child_stdin, "WINDOW1_BEGIN");
                        window_one_acked = true;
                    }
                    StabilityPhase::WindowTwo if !window_two_acked => {
                        write_stability_ack(&mut child_stdin, "WINDOW2_BEGIN");
                        window_two_acked = true;
                    }
                    StabilityPhase::Quiescent if !quiescent_acked => {
                        let soak_elapsed = soak_done_at
                            .expect("QUIESCENT_READY observed before SOAK_DONE")
                            .elapsed();
                        assert!(
                            soak_elapsed >= Duration::from_millis(250),
                            "quiescent sample occurred before 250ms post-soak: {soak_elapsed:?}"
                        );
                        write_stability_ack(&mut child_stdin, "QUIESCENT_READY");
                        quiescent_acked = true;
                    }
                    _ => {}
                }
            }
        }
        if let Some(status) = child.try_wait().expect("poll stability program") {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().ok();
            let status = child.wait().expect("wait after stability timeout");
            panic!(
                "stability program timed out after {STABILITY_TIMEOUT:?}: {status:?}; \
                 last_marker={last_marker:?} last_sample={last_sample:?} \
                 acks={{baseline:{baseline_acked},window1:{window_one_acked},\
                 window2:{window_two_acked},soak_done:{soak_done_acked},\
                 quiescent:{quiescent_acked}}}"
            );
        }
        thread::sleep(Duration::from_millis(20));
    };

    let stdout = stdout_reader.join().expect("join stability stdout reader");
    let stderr = stderr_reader.join().expect("join stability stderr reader");
    while let Ok(line) = line_rx.try_recv() {
        observe_stability_line(&line, &mut rounds);
    }
    assert!(
        status.success(),
        "Mamba stability program failed ({status:?})\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        rounds,
        (0_u32..100).collect::<BTreeSet<_>>(),
        "every stability round must emit exact success evidence"
    );

    let baseline_threads = samples
        .baseline_threads
        .unwrap_or_else(|| {
            panic!(
                "missing pre-soak OS thread sample; last_marker={last_marker:?} last_sample={last_sample:?}"
            )
        });
    let post_threads = samples
        .post_threads
        .unwrap_or_else(|| {
            panic!(
                "missing post-quiescence OS thread sample; last_marker={last_marker:?} last_sample={last_sample:?}"
            )
        });
    assert!(
        post_threads <= baseline_threads + 1,
        "worker/thread count did not quiesce: baseline={baseline_threads}, post={post_threads}"
    );
    assert!(
        samples.window_one_peak_rss > 0,
        "missing window-one RSS sample; last_marker={last_marker:?} last_sample={last_sample:?}"
    );
    assert!(
        samples.window_two_peak_rss > 0,
        "missing window-two RSS sample; last_marker={last_marker:?} last_sample={last_sample:?}"
    );
    let rss_limit = samples
        .window_one_peak_rss
        .saturating_mul(110)
        .saturating_div(100)
        .saturating_add(8 * MIB);
    assert!(
        samples.window_two_peak_rss <= rss_limit,
        "window-two RSS exceeded leak bound: window1={} window2={} limit={}",
        samples.window_one_peak_rss,
        samples.window_two_peak_rss,
        rss_limit
    );
    println!(
        "MAMBA-T1-FT-GATHER-STABILITY rounds=100 workers=8 baseline_threads={} post_threads={} window1_peak_rss={} window2_peak_rss={} digest={:016x}",
        baseline_threads,
        post_threads,
        samples.window_one_peak_rss,
        samples.window_two_peak_rss,
        stable_digest(stdout.as_bytes())
    );
}

#[test]
fn build_globals_dict_key_leak_free() {
    const NAME_COUNT: u32 = 20;
    // build_globals_dict inserts one implicit `__name__` dunder, and the
    // script's own `total` accumulator is itself a module-level global by
    // the time the loop reads it back (confirmed empirically: len(globals())
    // == ints + funcs + 2; the for-loop target `_` does not appear). Equal
    // int/function counts mean a regression isolated to either the id_ns
    // loop or the func_info loop alone still leaks NAME_COUNT keys/call, not
    // just one — comfortably clear of noise.
    const EXPECTED_LEN: u64 = NAME_COUNT as u64 * 2 + 2;

    let mut names_block = String::new();
    for i in 0..NAME_COUNT {
        names_block.push_str(&format!("g{i} = {i}\n"));
    }
    for i in 0..NAME_COUNT {
        names_block.push_str(&format!("def f{i}():\n    return {i}\n\n"));
    }
    let make_source = |iterations: u32| {
        format!(
            r#"{names_block}
total = 0
for _ in range({iterations}):
    total += len(globals())
print("DONE", total)
"#
        )
    };
    let parse_total = |run: &MeasuredRun| -> u64 {
        run.stdout
            .split_whitespace()
            .last()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| panic!("could not parse DONE total from stdout: {}", run.stdout))
    };

    let small = run_mamba_script(&make_source(100), EFFICIENCY_TIMEOUT);
    assert_success("build_globals_dict_key_leak_free (small)", &small);
    let large = run_mamba_script(&make_source(50_000), EFFICIENCY_TIMEOUT);
    assert_success("build_globals_dict_key_leak_free (large)", &large);

    // Guard against a vacuous pass: if build_globals_dict ever degenerated
    // to an empty or wrong-count dict, there would be nothing left to leak
    // and the RSS check below would trivially "pass" for the wrong reason.
    // Pin the exact per-call key count so that failure mode is caught here.
    let small_total = parse_total(&small);
    let large_total = parse_total(&large);
    assert_eq!(
        small_total,
        100 * EXPECTED_LEN,
        "small run: unexpected globals() key count (build_globals_dict may be degenerate)"
    );
    assert_eq!(
        large_total,
        50_000 * EXPECTED_LEN,
        "large run: unexpected globals() key count (build_globals_dict may be degenerate)"
    );

    // Each globals() call fabricates one Str key per exposed name inside
    // build_globals_dict; an unreleased key leaks one heap allocation per
    // name per call, so peak RSS grows roughly linearly with call count
    // instead of plateauing. Fixed slack (not a ratio) because both runs
    // share the same fixed interpreter-startup RSS floor — only true
    // per-call growth should show up in the delta.
    let rss_limit = small.peak_rss_bytes.saturating_add(24 * MIB);
    assert!(
        large.peak_rss_bytes <= rss_limit,
        "peak RSS grew with globals() call count — build_globals_dict key \
         leak suspected: small(100 calls)={} bytes, large(50_000 calls)={} \
         bytes, limit={} bytes",
        small.peak_rss_bytes,
        large.peak_rss_bytes,
        rss_limit
    );
    println!(
        "MAMBA-T1-BUILD-GLOBALS-DICT-KEY-LEAK-FREE names_per_call={} iterations_small=100 iterations_large=50000 rss_small={} rss_large={}",
        EXPECTED_LEN, small.peak_rss_bytes, large.peak_rss_bytes
    );
}
// </HANDWRITE>

// <HANDWRITE gap="missing-generator:logic" tracker="#1942" reason="logic section in mamba_core_semantics_ec.rs is hand-written pending codegen support">
#[test]
fn to_thread_gather_efficiency() {
    let logical_cpus = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(0);
    assert!(
        logical_cpus >= 4,
        "UNSUPPORTED HOST: MAMBA-T1-FT-GATHER-EFFICIENCY requires at least four logical CPUs; observed {logical_cpus}"
    );

    let serial_source = r#"
def cpu_work(seed: int, work: int) -> int:
    return sum((i ^ seed) * 3 for i in range(work))

results = [
    cpu_work(0, 600000),
    cpu_work(1, 600000),
    cpu_work(2, 600000),
    cpu_work(3, 600000),
]
print("RESULTS", results[0], results[1], results[2], results[3])
"#;
    let parallel_source = r#"
import asyncio

def cpu_work(seed: int, work: int) -> int:
    return sum((i ^ seed) * 3 for i in range(work))

async def main() -> None:
    results = await asyncio.gather(
        asyncio.to_thread(cpu_work, 0, 600000),
        asyncio.to_thread(cpu_work, 1, 600000),
        asyncio.to_thread(cpu_work, 2, 600000),
        asyncio.to_thread(cpu_work, 3, 600000),
    )
    print("RESULTS", results[0], results[1], results[2], results[3])

asyncio.run(main())
"#;

    let serial = run_mamba_script(serial_source, EFFICIENCY_TIMEOUT);
    assert_success("serial Mamba efficiency control", &serial);
    let parallel = run_mamba_script(parallel_source, EFFICIENCY_TIMEOUT);
    assert_success("parallel Mamba to_thread gather", &parallel);
    assert_eq!(
        parallel.stdout, serial.stdout,
        "parallel results differ from serial control"
    );

    let serial_wall = serial.wall.as_secs_f64();
    let parallel_wall = parallel.wall.as_secs_f64();
    let speedup = serial_wall / parallel_wall;
    let parallel_cpu_ratio = parallel.cpu_seconds / parallel_wall;
    assert!(
        speedup >= 1.50,
        "parallel wall speedup below 1.50x: serial={serial_wall:.6}s parallel={parallel_wall:.6}s speedup={speedup:.3}x"
    );
    assert!(
        parallel_cpu_ratio >= 1.50,
        "parallel process CPU/wall below 1.50: cpu={:.6}s wall={parallel_wall:.6}s ratio={parallel_cpu_ratio:.3}",
        parallel.cpu_seconds
    );
    let parallel_rss_limit = serial
        .peak_rss_bytes
        .saturating_mul(125)
        .saturating_div(100)
        .saturating_add(16 * MIB);
    assert!(
        parallel.peak_rss_bytes <= parallel_rss_limit,
        "parallel peak RSS exceeded bound: serial={} parallel={} limit={}",
        serial.peak_rss_bytes,
        parallel.peak_rss_bytes,
        parallel_rss_limit
    );

    println!(
        "MAMBA-T1-FT-GATHER-EFFICIENCY logical_cpus={} serial_wall={:.6} parallel_wall={:.6} parallel_cpu={:.6} cpu_wall={:.3} serial_peak_rss={} parallel_peak_rss={} result_digest={:016x} speedup={:.3}",
        logical_cpus,
        serial_wall,
        parallel_wall,
        parallel.cpu_seconds,
        parallel_cpu_ratio,
        serial.peak_rss_bytes,
        parallel.peak_rss_bytes,
        stable_digest(parallel.stdout.as_bytes()),
        speedup
    );
}
// </HANDWRITE>

// <HANDWRITE gap="missing-generator:unit-test" tracker="#1942" reason="unit-test section in mamba_core_semantics_ec.rs is hand-written for WI #1942 EC">
#[test]
fn type_wall_conformance_determinism() {
    let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/t1_type_wall_denominator/manifest.toml");
    let denom_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/t1_type_wall_denominator/denominator.txt");
    let baseline_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "external-contracts/evidence/mamba-t1-type-wall-conformance-determinism-baseline.json",
    );

    // 1. Verify manifest.toml and denominator.txt
    let (denom_rows, manifest_digest) =
        verify_type_wall_manifest_and_denominator(&manifest_path, &denom_path);

    // 2. Load and validate baseline JSON evidence artifact
    let baseline = load_type_wall_baseline(&baseline_path);
    assert_eq!(
        baseline.denominator_sha256, manifest_digest,
        "Baseline denominator_sha256 mismatch"
    );
    assert_eq!(
        baseline.manifest_path,
        "projects/mamba/tests/governance/gates/t1_type_wall_denominator/manifest.toml",
        "Baseline manifest_path mismatch"
    );
    assert_eq!(baseline.row_count, 7407, "Baseline row_count mismatch");
    assert_eq!(
        baseline.command,
        "cargo test -p mamba --release --test cpython_ported_integration cpython_ported::gen::_type -- --nocapture",
        "Baseline command mismatch"
    );
    assert_eq!(
        baseline.source_revision, "9113cc02e6f38321092e308726bf968b4c709438",
        "Baseline source_revision mismatch"
    );
    assert_eq!(
        baseline.failure_count,
        baseline.allowed_failing_paths.len(),
        "Baseline failure_count must equal cardinality of allowed_failing_paths"
    );

    // Validate capture_timestamp as RFC3339 UTC timestamp per Item 5
    let parsed_dt = chrono::DateTime::parse_from_rfc3339(&baseline.capture_timestamp)
        .expect("Baseline capture_timestamp must be valid RFC3339 timestamp");
    assert_eq!(
        parsed_dt.offset().local_minus_utc(),
        0,
        "Baseline capture_timestamp offset must be UTC"
    );

    // Environment fingerprint & drift check before runs (Round 4 Tail Fix 4 & 5)
    let rustc_ver = get_rustc_version();
    let cargo_ver = get_cargo_version();
    let git_head = get_git_head_revision();

    assert_eq!(
        git_head, baseline.source_revision,
        "Git HEAD revision mismatch: expected {:?}, got {:?}",
        baseline.source_revision, git_head
    );
    assert_eq!(
        baseline.rustup_toolchain, rustc_ver,
        "Baseline rustup_toolchain mismatch: expected {:?}, got {:?}",
        rustc_ver, baseline.rustup_toolchain
    );
    assert_eq!(
        baseline.cargo_version, cargo_ver,
        "Baseline cargo_version mismatch: expected {:?}, got {:?}",
        cargo_ver, baseline.cargo_version
    );

    let env_fp_before = compute_type_wall_env_fingerprint(
        &manifest_digest,
        &baseline.source_revision,
        &rustc_ver,
        &cargo_ver,
        &baseline.command,
        "8",
    );
    assert_eq!(
        baseline.environment_fingerprint, env_fp_before,
        "Baseline environment_fingerprint mismatch: expected {}, got {}",
        env_fp_before, baseline.environment_fingerprint
    );

    // 2.5 Execute full and filtered --list subprocess preflights
    println!("Executing full-target --list preflight...");
    let (full_list_set, _full_list_raw) = execute_list_preflight(
        &[
            "test",
            "-p",
            "mamba",
            "--release",
            "--test",
            "cpython_ported_integration",
            "--",
            "--list",
        ],
        Duration::from_secs(1200),
    );
    assert_eq!(
        full_list_set.len(),
        13767,
        "Full target list count mismatch: expected 13767, got {}",
        full_list_set.len()
    );
    for d in &denom_rows {
        assert!(
            full_list_set.contains(d),
            "Denominator row {} missing from full target list",
            d
        );
    }

    println!("Executing filtered-target --list preflight...");
    let (filtered_list_set, _filtered_list_raw) = execute_list_preflight(
        &[
            "test",
            "-p",
            "mamba",
            "--release",
            "--test",
            "cpython_ported_integration",
            "cpython_ported::gen::_type",
            "--",
            "--list",
        ],
        Duration::from_secs(1200),
    );
    assert_eq!(
        filtered_list_set.len(),
        7407,
        "Filtered target list count mismatch: expected 7407, got {}",
        filtered_list_set.len()
    );
    assert_eq!(
        filtered_list_set, denom_rows,
        "Filtered target list set mismatch with denominator.txt set"
    );

    // 3. Execute 3 fresh isolated release conformance runs with per-run timeout >= 1200s & run-local capture
    let mut run_failing_sets: Vec<BTreeSet<String>> = Vec::new();
    let mut run_failing_counts: Vec<usize> = Vec::new();
    let mut last_captured_outcomes: Vec<BTreeMap<String, String>> = Vec::new();
    let mut last_full_summaries: Vec<String> = Vec::new();

    for run_idx in 0..3 {
        println!(
            "Starting isolated release conformance run {}/3...",
            run_idx + 1
        );

        let (stdout, stderr, exit_code) =
            execute_conformance_run_with_timeout(Duration::from_secs(1200));

        let (denom_outcomes, failing_set, summary_str) =
            parse_and_verify_run_outcomes(&stdout, &stderr, &denom_rows, exit_code);

        // Run real observed outcomes map through production validator per Correction 8
        let (val_failing, val_count) = validate_type_wall_outcomes(
            &denom_outcomes,
            &denom_rows,
            &baseline.allowed_failing_paths,
            baseline.failure_count,
        )
        .expect("Real observed outcomes failed production validator");

        assert_eq!(val_count, 7407);

        run_failing_counts.push(val_failing.len());
        run_failing_sets.push(failing_set);
        last_captured_outcomes.push(denom_outcomes);
        last_full_summaries.push(summary_str);
    }

    // Environment fingerprint drift check after runs
    let git_head_after = get_git_head_revision();
    assert_eq!(
        git_head_after, baseline.source_revision,
        "Git HEAD revision drift"
    );

    let env_fp_after = compute_type_wall_env_fingerprint(
        &manifest_digest,
        &baseline.source_revision,
        &rustc_ver,
        &cargo_ver,
        &baseline.command,
        "8",
    );
    assert_eq!(
        env_fp_before, env_fp_after,
        "Environment fingerprint drift detected across test execution"
    );

    // 4. Assert exact equality of failing sets and counts across all 3 runs
    assert_eq!(
        run_failing_counts[0], run_failing_counts[1],
        "Run 1 and Run 2 failure counts differ: {} vs {}",
        run_failing_counts[0], run_failing_counts[1]
    );
    assert_eq!(
        run_failing_counts[1], run_failing_counts[2],
        "Run 2 and Run 3 failure counts differ: {} vs {}",
        run_failing_counts[1], run_failing_counts[2]
    );
    assert_eq!(
        run_failing_sets[0], run_failing_sets[1],
        "Run 1 and Run 2 failing sets differ"
    );
    assert_eq!(
        run_failing_sets[1], run_failing_sets[2],
        "Run 2 and Run 3 failing sets differ"
    );

    // 5. Assert baseline subset containment & failure count bound
    let post_fix_failing_set = &run_failing_sets[0];
    let post_fix_count = run_failing_counts[0];

    assert!(
        post_fix_count <= baseline.failure_count,
        "Post-fix failure count {} exceeds pre-fix baseline failure count {}",
        post_fix_count,
        baseline.failure_count
    );

    for path in post_fix_failing_set {
        assert!(
            baseline.allowed_failing_paths.contains(path),
            "Failing path {:?} is not in baseline allowed set",
            path
        );
    }

    // 8. Fail-closed mutation canaries over captured outcomes map using production validator
    let real_outcomes = &last_captured_outcomes[0];

    // Canary 1: Omission canary — remove 1 denominator row from outcome map and pass to validator
    let mut omitted_map = real_outcomes.clone();
    if let Some(first_key) = omitted_map.keys().next().cloned() {
        omitted_map.remove(&first_key);
    }
    let canary1_result = validate_type_wall_outcomes(
        &omitted_map,
        &denom_rows,
        &baseline.allowed_failing_paths,
        baseline.failure_count,
    );
    assert!(
        canary1_result.is_err(),
        "Fail-closed canary FAILED: omitted row was not rejected by production validator"
    );

    // Canary 2: Outcome-flip canary — flip 1 real denominator member's outcome from PASS to FAIL
    let mut flipped_map = real_outcomes.clone();
    let mut flipped_key = String::new();
    for (k, v) in real_outcomes {
        if v == "PASS" {
            flipped_key = k.clone();
            break;
        }
    }
    assert!(
        !flipped_key.is_empty(),
        "No passing key found to flip for canary 2"
    );
    flipped_map.insert(flipped_key, "FAIL".to_string());
    let canary2_result = validate_type_wall_outcomes(
        &flipped_map,
        &denom_rows,
        &baseline.allowed_failing_paths,
        baseline.failure_count,
    );
    assert!(
        canary2_result.is_err(),
        "Fail-closed canary FAILED: outcome-flip mutation was not rejected by production validator"
    );

    println!(
        "MAMBA-T1-TYPE-WALL-CONFORMANCE-DETERMINISM PASS runs=3 total=7407 failures={} baseline_failures={} summary={:?}",
        post_fix_count,
        baseline.failure_count,
        last_full_summaries[0]
    );
}

fn verify_type_wall_manifest_and_denominator(
    manifest_path: &std::path::Path,
    denom_path: &std::path::Path,
) -> (BTreeSet<String>, String) {
    assert!(
        manifest_path.is_file(),
        "Missing manifest.toml at {}",
        manifest_path.display()
    );
    assert!(
        denom_path.is_file(),
        "Missing denominator.txt at {}",
        denom_path.display()
    );

    let manifest_raw = std::fs::read_to_string(manifest_path).expect("read manifest.toml");
    let manifest_toml: toml::Value = manifest_raw.parse().expect("parse manifest.toml");

    let row_count = manifest_toml
        .get("row_count")
        .and_then(|v| v.as_integer())
        .expect("manifest row_count") as usize;
    assert_eq!(row_count, 7407, "manifest row_count must be 7407");

    let expected_digest = manifest_toml
        .get("denominator_sha256")
        .and_then(|v| v.as_str())
        .expect("manifest denominator_sha256")
        .to_string();
    assert_eq!(
        expected_digest,
        "eb45a673ca92c766c1df6596592aa226fae56ab81f8f27fea47e1168743eae28"
    );

    let denom_bytes = std::fs::read(denom_path).expect("read denominator.txt");
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&denom_bytes);
    let actual_digest = format!("{:x}", hasher.finalize());
    assert_eq!(
        actual_digest, expected_digest,
        "denominator.txt sha256 mismatch"
    );

    let denom_str = String::from_utf8_lossy(&denom_bytes);
    let denom_rows: BTreeSet<String> = denom_str
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        denom_rows.len(),
        7407,
        "denominator.txt must contain exactly 7407 unique lines"
    );

    (denom_rows, expected_digest)
}

struct TypeWallBaselineEvidence {
    source_revision: String,
    manifest_path: String,
    denominator_sha256: String,
    row_count: usize,
    command: String,
    capture_timestamp: String,
    rustup_toolchain: String,
    cargo_version: String,
    environment_fingerprint: String,
    allowed_failing_paths: BTreeSet<String>,
    failure_count: usize,
}

fn load_type_wall_baseline(baseline_path: &std::path::Path) -> TypeWallBaselineEvidence {
    assert!(
        baseline_path.is_file(),
        "Missing baseline artifact at {}",
        baseline_path.display()
    );

    let raw = std::fs::read_to_string(baseline_path).expect("read baseline json");
    let val: serde_json::Value = serde_json::from_str(&raw).expect("parse baseline json");

    let manifest_path = val
        .get("manifest_path")
        .and_then(|v| v.as_str())
        .expect("baseline manifest_path")
        .to_string();
    let denominator_sha256 = val
        .get("denominator_sha256")
        .or_else(|| val.get("manifest_digest"))
        .and_then(|v| v.as_str())
        .expect("baseline denominator_sha256")
        .to_string();
    let row_count = val
        .get("row_count")
        .or_else(|| val.get("manifest_row_count"))
        .and_then(|v| v.as_u64())
        .expect("baseline row_count") as usize;
    let command = val
        .get("command")
        .and_then(|v| v.as_str())
        .expect("baseline command")
        .to_string();
    let capture_timestamp = val
        .get("capture_timestamp")
        .and_then(|v| v.as_str())
        .expect("baseline capture_timestamp")
        .to_string();
    let rustup_toolchain = val
        .get("rustup_toolchain")
        .and_then(|v| v.as_str())
        .expect("baseline rustup_toolchain")
        .to_string();
    let cargo_version = val
        .get("cargo_version")
        .and_then(|v| v.as_str())
        .expect("baseline cargo_version")
        .to_string();
    let environment_fingerprint = val
        .get("environment_fingerprint")
        .and_then(|v| v.as_str())
        .expect("baseline environment_fingerprint")
        .to_string();
    let source_revision = val
        .get("source_revision")
        .or_else(|| val.get("revision"))
        .and_then(|v| v.as_str())
        .expect("baseline source_revision")
        .to_string();
    let allowed_array = val
        .get("allowed_failing_paths")
        .or_else(|| val.get("full_normalized_allowed_failing_path_set"))
        .and_then(|v| v.as_array())
        .expect("baseline allowed_failing_paths");
    let allowed_failing_paths: BTreeSet<String> = allowed_array
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let failure_count = val
        .get("failure_count")
        .and_then(|v| v.as_u64())
        .expect("baseline failure_count") as usize;

    TypeWallBaselineEvidence {
        source_revision,
        manifest_path,
        denominator_sha256,
        row_count,
        command,
        capture_timestamp,
        rustup_toolchain,
        cargo_version,
        environment_fingerprint,
        allowed_failing_paths,
        failure_count,
    }
}

fn get_rustc_version() -> String {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("get rustc version");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn get_cargo_version() -> String {
    let output = Command::new("cargo")
        .arg("--version")
        .output()
        .expect("get cargo version");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn get_git_head_revision() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("failed to execute git rev-parse HEAD");
    assert!(
        output.status.success(),
        "git rev-parse HEAD exited with status {:?}",
        output.status
    );
    let rev = String::from_utf8(output.stdout)
        .expect("git rev-parse HEAD output is not valid UTF-8")
        .trim()
        .to_string();
    assert_eq!(
        rev.len(),
        40,
        "git rev-parse HEAD output must be exactly 40 hex characters, got {:?}",
        rev
    );
    assert!(
        rev.chars().all(|c| c.is_ascii_hexdigit()),
        "git rev-parse HEAD output must be valid hex characters, got {:?}",
        rev
    );
    rev
}

fn compute_type_wall_env_fingerprint(
    manifest_digest: &str,
    revision: &str,
    rustc_ver: &str,
    cargo_ver: &str,
    command: &str,
    threads: &str,
) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(manifest_digest.as_bytes());
    hasher.update(b":");
    hasher.update(revision.as_bytes());
    hasher.update(b":");
    hasher.update(rustc_ver.as_bytes());
    hasher.update(b":");
    hasher.update(cargo_ver.as_bytes());
    hasher.update(b":");
    hasher.update(command.as_bytes());
    hasher.update(b":");
    hasher.update(threads.as_bytes());
    format!("{:x}", hasher.finalize())
}

struct ProcessGroupGuard {
    pgid: i32,
    reaped: bool,
}

impl ProcessGroupGuard {
    fn kill_all(&mut self) {
        if !self.reaped && self.pgid > 0 {
            unsafe {
                let _ = libc::kill(-self.pgid, libc::SIGTERM);
                thread::sleep(Duration::from_millis(100));
                let _ = libc::kill(-self.pgid, libc::SIGKILL);
            }
            self.reaped = true;
        }
    }
}

impl Drop for ProcessGroupGuard {
    fn drop(&mut self) {
        self.kill_all();
    }
}

static RUN_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn execute_list_preflight(args: &[&str], timeout: Duration) -> (BTreeSet<String>, String) {
    let package_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let run_seq = RUN_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let pid = std::process::id();
    let epoch_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let stdout_path = std::env::temp_dir().join(format!(
        "mamba_list_stdout_{pid}_{epoch_nanos}_{run_seq}.log"
    ));
    let stderr_path = std::env::temp_dir().join(format!(
        "mamba_list_stderr_{pid}_{epoch_nanos}_{run_seq}.log"
    ));

    let stdout_file =
        std::fs::File::create(&stdout_path).expect("create run-local --list stdout temp file");
    let stderr_file =
        std::fs::File::create(&stderr_path).expect("create run-local --list stderr temp file");

    use std::os::unix::process::CommandExt;
    let mut cmd = Command::new("cargo");
    cmd.args(args)
        .current_dir(&package_dir)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = std::fs::remove_file(&stdout_path);
            let _ = std::fs::remove_file(&stderr_path);
            panic!("spawn cargo test --list subprocess failed: {e}");
        }
    };
    let pgid = child.id() as i32;
    let mut pg_guard = ProcessGroupGuard {
        pgid,
        reaped: false,
    };

    let start = Instant::now();
    let exit_code = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                pg_guard.reaped = true;
                if let Some(code) = status.code() {
                    break code;
                } else {
                    let _ = std::fs::remove_file(&stdout_path);
                    let _ = std::fs::remove_file(&stderr_path);
                    panic!(
                        "cargo test --list subprocess terminated by signal: {:?}",
                        status
                    );
                }
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    pg_guard.kill_all();
                    let _ = child.wait();
                    let _ = std::fs::remove_file(&stdout_path);
                    let _ = std::fs::remove_file(&stderr_path);
                    panic!(
                        "cargo test --list subprocess execution timed out after {:?}",
                        timeout
                    );
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                pg_guard.kill_all();
                let _ = child.wait();
                let _ = std::fs::remove_file(&stdout_path);
                let _ = std::fs::remove_file(&stderr_path);
                panic!("Error waiting on cargo test --list subprocess: {e}");
            }
        }
    };

    let stdout_bytes = match std::fs::read(&stdout_path) {
        Ok(b) => b,
        Err(e) => {
            let _ = std::fs::remove_file(&stdout_path);
            let _ = std::fs::remove_file(&stderr_path);
            panic!("read --list stdout temp file failed: {e}");
        }
    };
    let stderr_bytes = match std::fs::read(&stderr_path) {
        Ok(b) => b,
        Err(e) => {
            let _ = std::fs::remove_file(&stdout_path);
            let _ = std::fs::remove_file(&stderr_path);
            panic!("read --list stderr temp file failed: {e}");
        }
    };
    let _ = std::fs::remove_file(&stdout_path);
    let _ = std::fs::remove_file(&stderr_path);

    if exit_code != 0 {
        let stderr_lossy = String::from_utf8_lossy(&stderr_bytes);
        panic!(
            "cargo test --list failed with exit code {}\nstderr:\n{}",
            exit_code, stderr_lossy
        );
    }

    let stdout = match String::from_utf8(stdout_bytes) {
        Ok(s) => s,
        Err(e) => panic!("cargo test --list stdout not valid UTF-8: {e}"),
    };
    let _stderr = match String::from_utf8(stderr_bytes) {
        Ok(s) => s,
        Err(e) => panic!("cargo test --list stderr not valid UTF-8: {e}"),
    };

    let mut names = BTreeSet::new();
    let mut summary_parsed: Option<(usize, usize)> = None;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        assert!(
            summary_parsed.is_none(),
            "Unexpected non-empty line after terminal summary in cargo test --list output: {:?}",
            line
        );

        if trimmed.ends_with(": test") {
            let name = trimmed[..trimmed.len() - 6].trim().to_string();
            assert!(
                !name.is_empty(),
                "Empty test name in cargo test --list output: {:?}",
                line
            );
            assert!(
                names.insert(name.clone()),
                "Duplicate test name observed in cargo test --list output: {}",
                name
            );
        } else if trimmed.ends_with(" benchmarks") && trimmed.contains(" tests, ") {
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() == 2 {
                let tests_part = parts[0].trim();
                let bench_part = parts[1].trim();

                let tests_tokens: Vec<&str> = tests_part.split_whitespace().collect();
                let bench_tokens: Vec<&str> = bench_part.split_whitespace().collect();

                if tests_tokens.len() == 2
                    && tests_tokens[1] == "tests"
                    && bench_tokens.len() == 2
                    && bench_tokens[1] == "benchmarks"
                {
                    let count: usize = tests_tokens[0]
                        .parse()
                        .expect("parse summary test count in cargo test --list");
                    let bench: usize = bench_tokens[0]
                        .parse()
                        .expect("parse summary benchmark count in cargo test --list");

                    summary_parsed = Some((count, bench));
                } else {
                    panic!(
                        "Unparseable/unexpected non-empty line in cargo test --list output: {:?}",
                        line
                    );
                }
            } else {
                panic!(
                    "Unparseable/unexpected non-empty line in cargo test --list output: {:?}",
                    line
                );
            }
        } else {
            panic!(
                "Unparseable/unexpected non-empty line in cargo test --list output: {:?}",
                line
            );
        }
    }

    let (summary_count, benchmarks_count) = summary_parsed.expect(
        "Missing terminal summary line ('<count> tests, 0 benchmarks') in cargo test --list output",
    );

    assert_eq!(
        benchmarks_count, 0,
        "Summary benchmark count must be 0, got {}",
        benchmarks_count
    );

    assert_eq!(
        summary_count,
        names.len(),
        "Summary test count ({summary_count}) does not match unique parsed test count ({})",
        names.len()
    );

    (names, stdout)
}

fn execute_conformance_run_with_timeout(timeout: Duration) -> (String, String, i32) {
    let package_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let run_seq = RUN_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let pid = std::process::id();
    let epoch_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let stdout_path = std::env::temp_dir().join(format!(
        "mamba_conf_stdout_{pid}_{epoch_nanos}_{run_seq}.log"
    ));
    let stderr_path = std::env::temp_dir().join(format!(
        "mamba_conf_stderr_{pid}_{epoch_nanos}_{run_seq}.log"
    ));

    let stdout_file =
        std::fs::File::create(&stdout_path).expect("create run-local stdout temp file");
    let stderr_file =
        std::fs::File::create(&stderr_path).expect("create run-local stderr temp file");

    use std::os::unix::process::CommandExt;
    let mut cmd = Command::new("cargo");
    cmd.args([
        "test",
        "-p",
        "mamba",
        "--release",
        "--test",
        "cpython_ported_integration",
        "cpython_ported::gen::_type",
        "--",
        "--nocapture",
    ])
    .env("RUST_TEST_THREADS", "8")
    .current_dir(&package_dir)
    .stdout(Stdio::from(stdout_file))
    .stderr(Stdio::from(stderr_file));

    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = std::fs::remove_file(&stdout_path);
            let _ = std::fs::remove_file(&stderr_path);
            panic!("spawn cargo test subprocess failed: {e}");
        }
    };
    let pgid = child.id() as i32;
    let mut pg_guard = ProcessGroupGuard {
        pgid,
        reaped: false,
    };

    let start = Instant::now();
    let exit_code = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                pg_guard.reaped = true;
                let code = status.code().unwrap_or_else(|| {
                    let _ = std::fs::remove_file(&stdout_path);
                    let _ = std::fs::remove_file(&stderr_path);
                    panic!("Subprocess terminated by signal: {:?}", status);
                });
                break code;
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    pg_guard.kill_all();
                    let _ = child.wait();
                    let _ = std::fs::remove_file(&stdout_path);
                    let _ = std::fs::remove_file(&stderr_path);
                    panic!("Subprocess execution timed out after {:?}", timeout);
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                pg_guard.kill_all();
                let _ = child.wait();
                let _ = std::fs::remove_file(&stdout_path);
                let _ = std::fs::remove_file(&stderr_path);
                panic!("Error waiting on subprocess: {e}");
            }
        }
    };

    let stdout = std::fs::read_to_string(&stdout_path).expect("read run-local stdout temp file");
    let stderr = std::fs::read_to_string(&stderr_path).expect("read run-local stderr temp file");
    let _ = std::fs::remove_file(&stdout_path);
    let _ = std::fs::remove_file(&stderr_path);

    assert!(
        exit_code == 0 || exit_code == 101,
        "Subprocess exited with unexpected exit code {}\nstdout snippet:\n{}\nstderr snippet:\n{}",
        exit_code,
        &stdout[..stdout.len().min(2000)],
        &stderr[..stderr.len().min(2000)]
    );

    (stdout, stderr, exit_code)
}

fn parse_and_verify_run_outcomes(
    stdout: &str,
    stderr: &str,
    denom_rows: &BTreeSet<String>,
    exit_code: i32,
) -> (BTreeMap<String, String>, BTreeSet<String>, String) {
    let mut full_observed_map: BTreeMap<String, String> = BTreeMap::new();
    let mut full_seen_keys: BTreeSet<String> = BTreeSet::new();

    // 1. Parse terminal lines for ENTIRE conformance suite into exact-name map per Tail Fix 1
    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(test_idx) = trimmed.find("test ") {
            let rest = &trimmed[test_idx + 5..];
            if let Some(dots_idx) = rest.find(" ... ") {
                let full_name = rest[..dots_idx].trim().to_string();
                let status_part = rest[dots_idx + 5..].trim();

                let outcome = if status_part.starts_with("ok") {
                    "PASS"
                } else if status_part.starts_with("FAILED") {
                    "FAIL"
                } else if status_part.starts_with("ignored") {
                    "IGNORED"
                } else {
                    continue;
                };

                if full_name.starts_with("cpython_") || denom_rows.contains(&full_name) {
                    assert!(
                        full_seen_keys.insert(full_name.clone()),
                        "Duplicate terminal line observed for test: {}",
                        full_name
                    );

                    full_observed_map.insert(full_name, outcome.to_string());
                }
            }
        }
    }

    let full_ok_count = full_observed_map.values().filter(|v| *v == "PASS").count();
    let full_failed_count = full_observed_map.values().filter(|v| *v == "FAIL").count();
    let full_ignored_count = full_observed_map
        .values()
        .filter(|v| *v == "IGNORED")
        .count();

    // 2. Parse and reconcile complete libtest summary against ENTIRE observed map per Tail Fix 1
    let summary_lines: Vec<&str> = stdout
        .lines()
        .chain(stderr.lines())
        .map(|l| l.trim())
        .filter(|l| l.starts_with("test result:"))
        .collect();

    assert_eq!(
        summary_lines.len(),
        1,
        "Expected exactly 1 libtest summary line ('test result: ...'), found {}",
        summary_lines.len()
    );

    let summary = summary_lines[0];
    let passed_parsed =
        parse_summary_number(summary, "passed").expect("parse passed count from summary");
    let failed_parsed =
        parse_summary_number(summary, "failed").expect("parse failed count from summary");
    let ignored_parsed =
        parse_summary_number(summary, "ignored").expect("parse ignored count from summary");
    let measured_parsed =
        parse_summary_number(summary, "measured").expect("parse measured count from summary");
    let filtered_parsed =
        parse_summary_number(summary, "filtered").expect("parse filtered count from summary");

    assert_eq!(
        measured_parsed, 0,
        "Full suite summary measured count must be 0, got {}",
        measured_parsed
    );
    assert_eq!(
        filtered_parsed, 6360,
        "Full suite summary filtered count must be 6360, got {}",
        filtered_parsed
    );

    let executed_summary_total = passed_parsed + failed_parsed + ignored_parsed + measured_parsed;
    let full_observed_total = full_observed_map.len();
    assert_eq!(
        full_observed_total, executed_summary_total,
        "Sum of full observed terminal lines ({full_observed_total}) does not match executed summary total ({executed_summary_total})"
    );
    assert_eq!(
        executed_summary_total, 7407,
        "Executed summary total must be 7407, got {}",
        executed_summary_total
    );
    assert_eq!(
        executed_summary_total + filtered_parsed,
        13767,
        "Whole list total (executed + filtered) must be 13767, got {}",
        executed_summary_total + filtered_parsed
    );

    assert_eq!(
        passed_parsed, full_ok_count,
        "Full suite summary passed count {} does not match full observed ok count {}",
        passed_parsed, full_ok_count
    );
    assert_eq!(
        failed_parsed, full_failed_count,
        "Full suite summary failed count {} does not match full observed FAILED count {}",
        failed_parsed, full_failed_count
    );
    assert_eq!(
        ignored_parsed, full_ignored_count,
        "Full suite summary ignored count {} does not match full observed IGNORED count {}",
        ignored_parsed, full_ignored_count
    );

    if exit_code == 0 {
        assert_eq!(
            full_failed_count, 0,
            "Child exited 0 but libtest summary contains {} failures",
            full_failed_count
        );
    } else if exit_code == 101 {
        assert!(
            full_failed_count > 0,
            "Child exited 101 but libtest summary contains 0 failures"
        );
    }

    // 3. Separately project exact 7,407 denominator members and require exact bidirectional equality per Tail Fix 1
    let mut denom_observed_map: BTreeMap<String, String> = BTreeMap::new();
    let mut denom_failing_set: BTreeSet<String> = BTreeSet::new();

    for d in denom_rows {
        let outcome = full_observed_map.get(d).unwrap_or_else(|| {
            panic!(
                "Denominator member {} was not found in full suite observed output",
                d
            );
        });

        assert_ne!(
            outcome, "IGNORED",
            "Denominator member {} was marked ignored in libtest output",
            d
        );

        denom_observed_map.insert(d.clone(), outcome.clone());
        if outcome == "FAIL" {
            denom_failing_set.insert(d.clone());
        }
    }

    assert_eq!(
        denom_observed_map.len(),
        7407,
        "Denominator map count mismatch: expected 7407, got {}",
        denom_observed_map.len()
    );

    (denom_observed_map, denom_failing_set, summary.to_string())
}

fn parse_summary_number(summary: &str, label: &str) -> Option<usize> {
    for part in summary.split(';') {
        let trimmed = part.trim();
        if trimmed.contains(label) {
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            for i in 0..tokens.len() {
                if tokens[i] == label && i > 0 {
                    if let Ok(n) = tokens[i - 1].parse::<usize>() {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}

fn validate_type_wall_outcomes(
    outcomes: &BTreeMap<String, String>,
    denom_rows: &BTreeSet<String>,
    allowed_failing_paths: &BTreeSet<String>,
    max_failures: usize,
) -> Result<(BTreeSet<String>, usize), String> {
    if outcomes.len() != 7407 {
        return Err(format!("Expected 7407 outcomes, got {}", outcomes.len()));
    }
    for d in denom_rows {
        if !outcomes.contains_key(d) {
            return Err(format!("Missing denominator key {}", d));
        }
    }
    for k in outcomes.keys() {
        if !denom_rows.contains(k) {
            return Err(format!("Extra non-denominator key in outcomes: {}", k));
        }
    }

    let mut failing = BTreeSet::new();
    for (k, v) in outcomes {
        if v == "FAIL" {
            if !allowed_failing_paths.contains(k) {
                return Err(format!("Unallowed failing path {}", k));
            }
            failing.insert(k.clone());
        } else if v != "PASS" {
            return Err(format!("Invalid outcome status {}", v));
        }
    }

    if failing.len() > max_failures {
        return Err(format!(
            "Failure count {} exceeds max {}",
            failing.len(),
            max_failures
        ));
    }

    Ok((failing, outcomes.len()))
}
// </HANDWRITE>
