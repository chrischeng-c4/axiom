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

    // Verify historical baseline internal consistency
    let historical_env_fp = compute_type_wall_env_fingerprint(
        &manifest_digest,
        &baseline.source_revision,
        &baseline.rustup_toolchain,
        &baseline.cargo_version,
        &baseline.command,
        "8",
    );
    assert_eq!(
        baseline.environment_fingerprint, historical_env_fp,
        "Baseline environment_fingerprint mismatch: expected {}, got {}",
        historical_env_fp, baseline.environment_fingerprint
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

    // Live exact toolchain assertions & per-run current environment fingerprint & revision tracking immediately before runs
    let rustc_ver = get_rustc_version();
    let cargo_ver = get_cargo_version();
    assert_eq!(
        rustc_ver, baseline.rustup_toolchain,
        "Live rustc version mismatch with baseline rustup_toolchain: expected {}, got {}",
        baseline.rustup_toolchain, rustc_ver
    );
    assert_eq!(
        cargo_ver, baseline.cargo_version,
        "Live cargo version mismatch with baseline cargo_version: expected {}, got {}",
        baseline.cargo_version, cargo_ver
    );

    let git_head_before = get_git_head_revision();

    let per_run_env_fp_before = compute_type_wall_env_fingerprint(
        &manifest_digest,
        &git_head_before,
        &rustc_ver,
        &cargo_ver,
        &baseline.command,
        "8",
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

        // Freshly re-observe and compare full current-run tuple after execution
        let (fresh_denom_rows, fresh_manifest_digest) =
            verify_type_wall_manifest_and_denominator(&manifest_path, &denom_path);
        let fresh_git_head = get_git_head_revision();
        let fresh_rustc_ver = get_rustc_version();
        let fresh_cargo_ver = get_cargo_version();
        let fresh_command = &baseline.command;
        let fresh_threads = "8";

        assert_eq!(
            fresh_denom_rows,
            denom_rows,
            "Denominator set drift after run {}",
            run_idx + 1
        );
        assert_eq!(
            fresh_manifest_digest,
            manifest_digest,
            "Manifest/denominator digest drift after run {}",
            run_idx + 1
        );
        assert_eq!(
            fresh_git_head,
            git_head_before,
            "Git HEAD revision drift after run {}",
            run_idx + 1
        );
        assert_eq!(
            fresh_rustc_ver,
            rustc_ver,
            "rustc version drift after run {}",
            run_idx + 1
        );
        assert_eq!(
            fresh_cargo_ver,
            cargo_ver,
            "cargo version drift after run {}",
            run_idx + 1
        );

        let fresh_env_fp = compute_type_wall_env_fingerprint(
            &fresh_manifest_digest,
            &fresh_git_head,
            &fresh_rustc_ver,
            &fresh_cargo_ver,
            fresh_command,
            fresh_threads,
        );
        assert_eq!(
            fresh_env_fp,
            per_run_env_fp_before,
            "Environment fingerprint drift after run {}",
            run_idx + 1
        );

        run_failing_counts.push(val_failing.len());
        run_failing_sets.push(failing_set);
        last_captured_outcomes.push(denom_outcomes);
        last_full_summaries.push(summary_str);
    }

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

// <HANDWRITE gap="missing-generator:unit-test" tracker="#2010" reason="oracle_hierarchy_and_result_identity test in mamba_core_semantics_ec.rs">
fn validate_cpython_executable_path_and_version(exe_path: &str, expected_ver_substring: &str) {
    let path = std::path::Path::new(exe_path);
    if !path.is_file() {
        panic!("CPython executable missing or invalid: {exe_path}");
    }
    let output = Command::new(path)
        .arg("--version")
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute CPython --version for {exe_path}: {e}"));
    if !output.status.success() {
        panic!(
            "CPython --version exited with non-zero status {:?} for {exe_path}",
            output.status
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout} {stderr}");
    if !combined.contains(expected_ver_substring) {
        panic!("CPython version mismatch for {exe_path}: expected substring {expected_ver_substring:?}, got {combined:?}");
    }
}

fn validate_oracle_version(oracle_kind: &str, oracle_version: &str) -> Result<(), String> {
    let ver = oracle_version.trim();
    if ver.is_empty() {
        return Err("oracle_version is empty".to_string());
    }
    match oracle_kind {
        "cpython312_identity" => {
            if !ver.starts_with("Python 3.12") {
                return Err(format!(
                    "cpython312_identity requires oracle_version starting with 'Python 3.12', got '{ver}'"
                ));
            }
        }
        "cpython313t_identity" => {
            if !ver.starts_with("Python 3.13") {
                return Err(format!(
                    "cpython313t_identity requires oracle_version starting with 'Python 3.13', got '{ver}'"
                ));
            }
        }
        "property" | "force_typed_expected" => {
            if ver.contains("Python 2.")
                || ver.contains("unknown")
                || ver.contains("invalid")
                || (!ver.starts_with("Python 3.") && !ver.starts_with("Mamba "))
            {
                return Err(format!(
                    "unsupported oracle_version '{ver}' for oracle_kind '{oracle_kind}'"
                ));
            }
        }
        unknown => {
            return Err(format!("unknown oracle_kind '{unknown}'"));
        }
    }
    Ok(())
}

fn is_self_oracle_command(oracle_kind: &str, oracle_cmd: &str, sut_cmd: &str) -> bool {
    if oracle_cmd.trim() == sut_cmd.trim() {
        return true;
    }
    if oracle_cmd.contains("mamba_core_semantics_ec") || sut_cmd.contains("mamba_core_semantics_ec")
    {
        return true;
    }
    if (oracle_kind == "cpython312_identity" || oracle_kind == "cpython313t_identity")
        && (oracle_cmd.starts_with("mamba") || oracle_cmd.contains(" mamba "))
    {
        return true;
    }
    false
}

#[derive(Clone, Debug)]
struct RawCaseRow {
    case_id: String,
    channel: String,
    tier1_dimension: String,
    oracle_kind: String,
    oracle_executable: String,
    oracle_version: String,
    oracle_command: String,
    sut_command: String,
    expected_divergence_class: String,
    expected_outcome_kind: String,
    expected_probe_anchor: String,
    expected_result_channel: String,
    expected_terminal_classification: String,
    diagnostic_class: Option<String>,
    diagnostic_span: Option<String>,
    fixture_or_probe_path: String,
    mamba_binary_sha256: String,
    mamba_git_sha: String,
    platform: String,
    source_set: String,
    source_identity: String,
    probe_id: String,
    sample_role: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ExecutedCommandEvidence {
    exit_code: Option<i32>,
    signal: Option<i32>,
    timed_out: bool,
    stdout_sha256: String,
    stderr_sha256: String,
    stdout_text: String,
    stderr_text: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ObservedRowRecord {
    case_id: String,
    channel: String,
    tier1_dimension: String,
    derived_classification: String,
    mamba_binary_sha256: String,
    mamba_git_sha: String,
    platform: String,
    oracle_executable: String,
    oracle_version: String,
    inventory_sha256: String,
    oracle_ev: ExecutedCommandEvidence,
    sut_ev: ExecutedCommandEvidence,
}

fn run_command_with_evidence(
    cmd_str: &str,
    cwd: &std::path::Path,
    timeout: Duration,
) -> ExecutedCommandEvidence {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd_str)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn sh -c '{cmd_str}' (cwd={}): {e}", cwd.display()));

    let pid = child.id() as libc::pid_t;

    let mut stdout_pipe = child.stdout.take().expect("capture stdout");
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout_pipe.read_to_end(&mut bytes).ok();
        bytes
    });

    let mut stderr_pipe = child.stderr.take().expect("capture stderr");
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr_pipe.read_to_end(&mut bytes).ok();
        bytes
    });

    let completion = Arc::new((Mutex::new(false), Condvar::new()));
    let timed_out_flag = Arc::new(AtomicBool::new(false));
    let watchdog_completion = Arc::clone(&completion);
    let watchdog_timed_out = Arc::clone(&timed_out_flag);

    let watchdog = thread::spawn(move || {
        let (lock, wake) = &*watchdog_completion;
        let done = lock.lock().expect("watchdog lock");
        let (done, wait) = wake
            .wait_timeout_while(done, timeout, |d| !*d)
            .expect("watchdog wait");
        if !*done && wait.timed_out() {
            watchdog_timed_out.store(true, Ordering::SeqCst);
            unsafe {
                libc::kill(pid, libc::SIGKILL);
            }
        }
    });

    let mut raw_status = 0;
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    let _ = unsafe { libc::wait4(pid, &mut raw_status, 0, &mut usage) };

    {
        let (lock, wake) = &*completion;
        *lock.lock().expect("completion lock") = true;
        wake.notify_all();
    }
    watchdog.join().ok();

    let status = ExitStatus::from_raw(raw_status);
    let stdout_bytes = stdout_reader.join().unwrap_or_default();
    let stderr_bytes = stderr_reader.join().unwrap_or_default();

    use sha2::{Digest, Sha256};
    let mut stdout_hasher = Sha256::new();
    stdout_hasher.update(&stdout_bytes);
    let stdout_sha256 = format!("{:x}", stdout_hasher.finalize());

    let mut stderr_hasher = Sha256::new();
    stderr_hasher.update(&stderr_bytes);
    let stderr_sha256 = format!("{:x}", stderr_hasher.finalize());

    ExecutedCommandEvidence {
        exit_code: status.code(),
        signal: status.signal(),
        timed_out: timed_out_flag.load(Ordering::SeqCst),
        stdout_sha256,
        stderr_sha256,
        stdout_text: String::from_utf8_lossy(&stdout_bytes).into_owned(),
        stderr_text: String::from_utf8_lossy(&stderr_bytes).into_owned(),
    }
}

fn parse_exact_manifest_case_identity(source_id: &str) -> Result<(&str, &str), String> {
    const CASE_DELIM: &str = "#case=";
    if source_id.matches(CASE_DELIM).count() != 1 {
        return Err(format!(
            "manifest identity must contain exactly one '{CASE_DELIM}' delimiter: '{source_id}'"
        ));
    }
    let idx = source_id
        .find(CASE_DELIM)
        .ok_or_else(|| format!("missing delimiter '{CASE_DELIM}' in '{source_id}'"))?;
    let manifest_path = &source_id[..idx];
    let case_name = &source_id[idx + CASE_DELIM.len()..];

    if manifest_path.contains('#') || case_name.contains('#') {
        return Err(format!(
            "manifest identity parts must not contain '#': '{source_id}'"
        ));
    }

    if !manifest_path.ends_with(".toml") {
        return Err(format!(
            "manifest identity path must end with '.toml': '{manifest_path}'"
        ));
    }

    if manifest_path.contains('\\') {
        return Err(format!(
            "manifest path in identity must not contain backslashes: '{manifest_path}'"
        ));
    }

    if manifest_path.starts_with('/') {
        return Err(format!(
            "manifest path in identity must be relative, got absolute: '{manifest_path}'"
        ));
    }

    for seg in manifest_path.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." {
            return Err(format!(
                "manifest path in identity contains invalid path segment ('', '.', or '..'): '{manifest_path}'"
            ));
        }
    }

    let p = std::path::Path::new(manifest_path);
    if p.is_absolute() {
        return Err(format!(
            "manifest path in identity must be relative, got absolute: '{manifest_path}'"
        ));
    }

    for comp in p.components() {
        match comp {
            std::path::Component::Normal(_) => {}
            _ => {
                return Err(format!(
                    "manifest path in identity contains invalid path component (absolute, '.', or '..'): '{manifest_path}'"
                ));
            }
        }
    }

    if case_name.trim().is_empty() || case_name != case_name.trim() {
        return Err(format!(
            "manifest case name must not be empty or have surrounding whitespace: '{case_name}'"
        ));
    }

    if case_name.contains('/') || case_name.contains('\\') {
        return Err(format!(
            "manifest case name must not contain path separators: '{case_name}'"
        ));
    }

    Ok((manifest_path, case_name))
}

fn validate_oracle_hierarchy_row_fields(
    row: &RawCaseRow,
    live_mamba_sha: &str,
    expected_git_sha: &str,
    live_platform: &str,
    discovered_sets: Option<&BTreeMap<String, DiscoveredSourceSet>>,
) -> Result<(), String> {
    const EMPTY_FILE_SHA256: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    if row.mamba_binary_sha256 == EMPTY_FILE_SHA256 {
        return Err(format!(
            "case '{}' mamba_binary_sha256 is the empty-file SHA-256",
            row.case_id
        ));
    }
    if live_mamba_sha != row.mamba_binary_sha256 {
        return Err(format!(
            "case '{}' mamba_binary_sha256 mismatch: case expects {}, live is {}",
            row.case_id, row.mamba_binary_sha256, live_mamba_sha
        ));
    }
    if expected_git_sha != row.mamba_git_sha {
        return Err(format!(
            "case '{}' mamba_git_sha mismatch: case expects {}, live is {}",
            row.case_id, row.mamba_git_sha, expected_git_sha
        ));
    }
    if live_platform != row.platform {
        return Err(format!(
            "case '{}' platform mismatch: case expects {}, live is {}",
            row.case_id, row.platform, live_platform
        ));
    }

    match row.oracle_kind.as_str() {
        "cpython312_identity" | "cpython313t_identity" | "property" | "force_typed_expected" => {}
        unknown => {
            return Err(format!(
                "unknown oracle_kind '{unknown}' for case '{}'",
                row.case_id
            ))
        }
    }

    validate_oracle_version(&row.oracle_kind, &row.oracle_version)?;

    match row.expected_divergence_class.as_str() {
        "none"
        | "force_typed_compile_reject"
        | "force_typed_runtime_reject"
        | "memory_leak_bound"
        | "perf_bound"
        | "thread_quiescence_bound" => {}
        unknown => {
            return Err(format!(
                "unknown expected_divergence_class '{unknown}' for case '{}'",
                row.case_id
            ))
        }
    }

    match row.expected_outcome_kind.as_str() {
        "ok" | "compile_error" | "runtime_error" | "property_red" => {}
        unknown => {
            return Err(format!(
                "unknown expected_outcome_kind '{unknown}' for case '{}'",
                row.case_id
            ))
        }
    }

    if row.channel != row.expected_result_channel {
        return Err(format!(
            "channel mismatch for case '{}': channel={} expected_result_channel={}",
            row.case_id, row.channel, row.expected_result_channel
        ));
    }

    match row.tier1_dimension.as_str() {
        "behavior" | "stability" | "efficiency" => {}
        unknown => {
            return Err(format!(
                "unknown tier1_dimension '{unknown}' for case '{}'",
                row.case_id
            ));
        }
    }

    match row.channel.as_str() {
        "compile" | "behavior" | "concurrency" | "performance" => {}
        unknown => {
            return Err(format!(
                "unknown channel '{unknown}' for case '{}'",
                row.case_id
            ));
        }
    }

    match row.expected_terminal_classification.as_str() {
        "green" | "intentional_red" => {}
        unknown => {
            return Err(format!(
                "unknown expected_terminal_classification '{unknown}' for case '{}'",
                row.case_id
            ));
        }
    }

    if row.sample_role != row.expected_terminal_classification {
        return Err(format!(
            "declaration mismatch: sample_role '{}' does not match expected_terminal_classification '{}' for case '{}'",
            row.sample_role, row.expected_terminal_classification, row.case_id
        ));
    }

    if row.fixture_or_probe_path.trim().is_empty() {
        return Err(format!(
            "fixture_or_probe_path is empty for case '{}'",
            row.case_id
        ));
    }

    // Verify fixture file existence on disk
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir);
    let fixture_path = if std::path::Path::new(&row.fixture_or_probe_path).is_absolute() {
        std::path::PathBuf::from(&row.fixture_or_probe_path)
    } else {
        repo_root.join(&row.fixture_or_probe_path)
    };
    if !fixture_path.exists() {
        return Err(format!(
            "declared fixture_or_probe_path does not exist on disk for case '{}': {}",
            row.case_id,
            fixture_path.display()
        ));
    }

    // Reject shell control, comment, and substitution syntax in SUT commands
    if row.sut_command.contains('#')
        || row.sut_command.contains('$')
        || row.sut_command.contains('`')
        || row.sut_command.contains(';')
        || row.sut_command.contains('&')
        || row.sut_command.contains('|')
        || row.sut_command.contains('<')
        || row.sut_command.contains('>')
        || row.sut_command.contains('\n')
        || row.sut_command.contains('\r')
        || row.sut_command.contains('(')
        || row.sut_command.contains(')')
    {
        return Err(format!(
            "shell control/comment/substitution syntax rejected in SUT command for case '{}': {}",
            row.case_id, row.sut_command
        ));
    }

    // Require real SUT execution: sut_command must NOT be a synthetic print-only command
    if row.sut_command.contains("python -c")
        || row.sut_command.contains("python3 -c")
        || row.sut_command.contains("python3.12 -c")
        || row.sut_command.contains("python3.13t -c")
        || row.sut_command.contains("-c \"import sys; print")
        || row.sut_command.contains("-c \"print(")
        || row.sut_command.contains("echo ")
    {
        return Err(format!(
            "synthetic print-only SUT command rejected for case '{}': {}",
            row.case_id, row.sut_command
        ));
    }

    // Require SUT argv[0] to be bound to live Mamba executable
    let sut_tokens: Vec<&str> = row.sut_command.split_whitespace().collect();
    if sut_tokens.is_empty() {
        return Err(format!("sut_command is empty for case '{}'", row.case_id));
    }
    let argv0 = sut_tokens[0];

    let live_mamba_bin = mamba_bin();
    let live_mamba_bin_path = std::path::Path::new(&live_mamba_bin);
    let live_mamba_name = live_mamba_bin_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("mamba");

    let resolved_sut_exe_path = if argv0 == "mamba" || argv0 == live_mamba_name {
        live_mamba_bin_path.to_path_buf()
    } else if std::path::Path::new(argv0).is_absolute() {
        std::path::PathBuf::from(argv0)
    } else {
        repo_root.join(argv0)
    };

    if !resolved_sut_exe_path.is_file() {
        return Err(format!(
            "SUT executable file does not exist at '{}' (argv[0]='{argv0}') for case '{}'",
            resolved_sut_exe_path.display(),
            row.case_id
        ));
    }

    let sut_exe_bytes = std::fs::read(&resolved_sut_exe_path).map_err(|e| {
        format!(
            "failed to read SUT executable at '{}' for case '{}': {e}",
            resolved_sut_exe_path.display(),
            row.case_id
        )
    })?;
    use sha2::{Digest, Sha256};
    let sut_exe_sha256 = format!("{:x}", Sha256::digest(&sut_exe_bytes));
    if sut_exe_sha256 != live_mamba_sha {
        return Err(format!(
            "SUT executable argv[0]='{argv0}' binary SHA-256 digest mismatch: expected {live_mamba_sha}, found {sut_exe_sha256} for case '{}'",
            row.case_id
        ));
    }

    // Require declared fixture/probe path as an exact parsed argument
    let declared_fixture_str = row.fixture_or_probe_path.trim();
    let norm_fixture_path = if std::path::Path::new(declared_fixture_str).is_absolute() {
        std::path::PathBuf::from(declared_fixture_str)
    } else {
        repo_root.join(declared_fixture_str)
    };

    let has_exact_fixture_arg = sut_tokens[1..].iter().any(|arg| {
        if *arg == declared_fixture_str {
            return true;
        }
        let norm_arg = if std::path::Path::new(arg).is_absolute() {
            std::path::PathBuf::from(arg)
        } else {
            repo_root.join(arg)
        };
        norm_arg == norm_fixture_path
    });

    if !has_exact_fixture_arg {
        return Err(format!(
            "SUT command arguments do not contain declared fixture/probe path as an exact parsed argument for case '{}': sut_cmd='{}', fixture='{}'",
            row.case_id, row.sut_command, row.fixture_or_probe_path
        ));
    }

    if is_self_oracle_command(&row.oracle_kind, &row.oracle_command, &row.sut_command) {
        return Err(format!(
            "self-oracle detected for case '{}': oracle_command='{}' sut_command='{}'",
            row.case_id, row.oracle_command, row.sut_command
        ));
    }

    if row.oracle_kind == "cpython312_identity" || row.oracle_kind == "cpython313t_identity" {
        let exe_name = std::path::Path::new(&row.oracle_executable)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if !row.oracle_command.contains(exe_name) && !row.oracle_command.contains("python3") {
            return Err(format!(
                "command drift: oracle_command '{}' does not match oracle_executable '{}' for case '{}'",
                row.oracle_command, row.oracle_executable, row.case_id
            ));
        }
    }

    // Require diagnostic_class and diagnostic_span if expected_outcome_kind is compile_error or force_typed
    if row.expected_outcome_kind == "compile_error"
        || row.expected_divergence_class.starts_with("force_typed")
    {
        if row
            .diagnostic_class
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            return Err(format!(
                "missing required diagnostic_class for compile_error case '{}'",
                row.case_id
            ));
        }
        if row
            .diagnostic_span
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            return Err(format!(
                "missing required diagnostic_span for compile_error case '{}'",
                row.case_id
            ));
        }
    }

    match row.source_set.as_str() {
        "ordinary_parity_corpus" | "tier1_ec_cases" | "tier1_gate_denominators" => {}
        unknown => {
            return Err(format!(
                "unknown source_set '{unknown}' for case '{}'",
                row.case_id
            ))
        }
    }

    match row.sample_role.as_str() {
        "green" | "intentional_red" => {}
        unknown => {
            return Err(format!(
                "unknown sample_role '{unknown}' for case '{}'",
                row.case_id
            ))
        }
    }

    if row.probe_id.trim().is_empty() {
        return Err(format!("probe_id is empty for case '{}'", row.case_id));
    }

    if row.source_identity.trim().is_empty() {
        return Err(format!(
            "source_identity is empty for case '{}'",
            row.case_id
        ));
    }

    if row.source_set == "ordinary_parity_corpus" {
        let is_manifest_identity =
            row.source_identity.contains("#case=") || row.source_identity.contains(".toml");
        let case_suffix = if is_manifest_identity {
            let (manifest_path, case_name) =
                parse_exact_manifest_case_identity(&row.source_identity).map_err(|e| {
                    format!(
                        "ordinary manifest identity invalid for case '{}': {e}",
                        row.case_id
                    )
                })?;
            if let Some(sets) = discovered_sets {
                if let Some(ordinary_set) = sets.get("ordinary_parity_corpus") {
                    if !ordinary_set.paths.iter().any(|p| p == manifest_path) {
                        return Err(format!(
                            "ordinary manifest path '{manifest_path}' is outside the discovered ordinary-manifest set for case '{}'",
                            row.case_id
                        ));
                    }
                }
            }
            case_name
        } else {
            std::path::Path::new(&row.source_identity)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
        };

        let fixture_stem = std::path::Path::new(&row.fixture_or_probe_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if case_suffix != fixture_stem {
            return Err(format!(
                "ordinary row source_identity case suffix '{case_suffix}' does not match fixture_or_probe_path filename stem '{fixture_stem}' for case '{}'",
                row.case_id
            ));
        }
    }

    Ok(())
}

fn classify_and_reconcile_row(
    row: &RawCaseRow,
    oracle_ev: &ExecutedCommandEvidence,
    sut_ev: &ExecutedCommandEvidence,
) -> Result<String, String> {
    if oracle_ev.timed_out {
        return Err(format!(
            "oracle_command timed out for case '{}'",
            row.case_id
        ));
    }
    if sut_ev.timed_out {
        return Err(format!("sut_command timed out for case '{}'", row.case_id));
    }

    let derived_cls = if row.expected_outcome_kind == "property_red" {
        if sut_ev.exit_code == Some(0) {
            return Err(format!(
                "property_red expected non-zero exit code, but sut_command exited 0 for case '{}'",
                row.case_id
            ));
        }
        if !sut_ev.stdout_text.contains(&row.expected_probe_anchor)
            && !sut_ev.stderr_text.contains(&row.expected_probe_anchor)
        {
            return Err(format!(
                "property_red anchor '{}' missing from SUT output for case '{}'",
                row.expected_probe_anchor, row.case_id
            ));
        }
        "intentional_red".to_string()
    } else {
        let oracle_anchor_found = oracle_ev.stdout_text.contains(&row.expected_probe_anchor)
            || oracle_ev.stderr_text.contains(&row.expected_probe_anchor);
        let sut_anchor_found = sut_ev.stdout_text.contains(&row.expected_probe_anchor)
            || sut_ev.stderr_text.contains(&row.expected_probe_anchor);

        if !oracle_anchor_found {
            return Err(format!(
                "oracle probe anchor '{}' missing for case '{}'",
                row.expected_probe_anchor, row.case_id
            ));
        }
        if !sut_anchor_found {
            return Err(format!(
                "SUT probe anchor '{}' missing for case '{}'",
                row.expected_probe_anchor, row.case_id
            ));
        }

        match row.expected_outcome_kind.as_str() {
            "ok" => {
                if oracle_ev.exit_code != Some(0) {
                    return Err(format!(
                        "oracle_command failed for ok case '{}'",
                        row.case_id
                    ));
                }
                if sut_ev.exit_code != Some(0) {
                    return Err(format!("sut_command failed for ok case '{}'", row.case_id));
                }
                match row.oracle_kind.as_str() {
                    "cpython312_identity" | "cpython313t_identity" | "force_typed_expected" => {
                        if oracle_ev.stdout_sha256 != sut_ev.stdout_sha256 {
                            return Err(format!(
                                "stdout hash mismatch for case '{}': oracle={}, sut={}",
                                row.case_id, oracle_ev.stdout_sha256, sut_ev.stdout_sha256
                            ));
                        }
                    }
                    "property" => {}
                    unknown => {
                        return Err(format!(
                            "unhandled oracle_kind '{unknown}' for case '{}'",
                            row.case_id
                        ));
                    }
                }
                "green".to_string()
            }
            "compile_error" | "runtime_error" => {
                if sut_ev.exit_code == Some(0) {
                    return Err(format!(
                        "expected error outcome but sut_command succeeded for case '{}'",
                        row.case_id
                    ));
                }
                "intentional_red".to_string()
            }
            unknown => {
                return Err(format!(
                    "unhandled expected_outcome_kind '{unknown}' for case '{}'",
                    row.case_id
                ));
            }
        }
    };

    if derived_cls != row.expected_terminal_classification {
        return Err(format!(
            "observed classification mismatch: derived '{derived_cls}' != expected_terminal_classification '{}' for case '{}'",
            row.expected_terminal_classification, row.case_id
        ));
    }

    if derived_cls != row.sample_role {
        return Err(format!(
            "observed classification mismatch: derived '{derived_cls}' != sample_role '{}' for case '{}'",
            row.sample_role, row.case_id
        ));
    }

    Ok(derived_cls)
}

fn reconcile_observed_dataset(
    records: &[ObservedRowRecord],
    expected_count: usize,
    expected_inventory_sha: &str,
) -> Result<(usize, usize, BTreeMap<String, usize>), String> {
    if records.is_empty() {
        return Err("zero observed execution records".to_string());
    }
    if records.len() != expected_count {
        return Err(format!(
            "observed record count {} != expected {}",
            records.len(),
            expected_count
        ));
    }

    let mut green_count = 0;
    let mut red_count = 0;
    let mut channel_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut dim_coverage: BTreeSet<(String, String)> = BTreeSet::new();

    for record in records {
        if record.inventory_sha256 != expected_inventory_sha {
            return Err(format!(
                "record inventory_sha256 mismatch for case '{}': expected {}, found {}",
                record.case_id, expected_inventory_sha, record.inventory_sha256
            ));
        }
        *channel_counts.entry(record.channel.clone()).or_default() += 1;
        dim_coverage.insert((
            record.tier1_dimension.clone(),
            record.derived_classification.clone(),
        ));

        match record.derived_classification.as_str() {
            "green" => green_count += 1,
            "intentional_red" => red_count += 1,
            other => {
                return Err(format!(
                    "unclassified record state '{other}' for case '{}'",
                    record.case_id
                ))
            }
        }
    }

    let required_dimensions = ["behavior", "stability", "efficiency"];
    let required_classifications = ["green", "intentional_red"];
    for dim in &required_dimensions {
        for cls in &required_classifications {
            if !dim_coverage.contains(&(dim.to_string(), cls.to_string())) {
                return Err(format!(
                    "missing required classification coverage '{cls}' for tier1_dimension '{dim}'"
                ));
            }
        }
    }

    let required_channels = ["compile", "behavior", "concurrency", "performance"];
    for ch in &required_channels {
        let count = channel_counts.get(*ch).copied().unwrap_or(0);
        if count == 0 {
            return Err(format!("zero count for required channel '{ch}'"));
        }
    }

    Ok((green_count, red_count, channel_counts))
}

#[derive(Clone, Debug, Default)]
struct DiscoveredSourceSet {
    name: String,
    paths: Vec<String>,
    identity_count: usize,
    sha256_digest: String,
    source_identities: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PinnedSourceSetRecord {
    name: String,
    paths: Vec<String>,
    identity_count: usize,
    sha256_digest: String,
}

fn parse_pinned_source_set_records_from_toml(
    val: &toml::Value,
) -> Result<Vec<PinnedSourceSetRecord>, String> {
    let source_sets_val = val
        .get("source_set_records")
        .or_else(|| val.get("pinned_source_sets"))
        .or_else(|| val.get("source_sets"));

    let arr = match source_sets_val {
        Some(v) => v
            .as_array()
            .ok_or_else(|| "source_sets in TOML is not an array".to_string())?,
        None => return Err("missing source_sets in TOML".to_string()),
    };

    let mut records = Vec::new();
    for item in arr {
        let obj = item
            .as_table()
            .ok_or_else(|| "source_set item in TOML is not a table/object".to_string())?;
        let name = obj
            .get("name")
            .or_else(|| obj.get("source_set"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing name/source_set in pinned source_sets record".to_string())?
            .to_string();
        let paths = obj
            .get("paths")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                "missing or invalid paths array in pinned source_sets record".to_string()
            })?
            .iter()
            .map(|p| {
                p.as_str()
                    .map(String::from)
                    .ok_or_else(|| "non-string in paths array".to_string())
            })
            .collect::<Result<Vec<String>, _>>()?;
        let identity_count = obj
            .get("identity_count")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| {
                "missing or invalid identity_count in pinned source_sets record".to_string()
            })? as usize;
        let sha256_digest = obj
            .get("sha256_digest")
            .or_else(|| obj.get("digest"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing sha256_digest in pinned source_sets record".to_string())?
            .to_string();

        records.push(PinnedSourceSetRecord {
            name,
            paths,
            identity_count,
            sha256_digest,
        });
    }

    Ok(records)
}

fn parse_pinned_source_set_records_from_json(
    val: &serde_json::Value,
) -> Result<Vec<PinnedSourceSetRecord>, String> {
    let source_sets_val = val
        .get("source_set_records")
        .or_else(|| val.get("pinned_source_sets"))
        .or_else(|| val.get("source_sets"));

    let arr = match source_sets_val {
        Some(v) => v
            .as_array()
            .ok_or_else(|| "source_sets in JSON is not an array".to_string())?,
        None => return Err("missing source_sets in JSON".to_string()),
    };

    let mut records = Vec::new();
    for item in arr {
        let obj = item
            .as_object()
            .ok_or_else(|| "source_set item in JSON is not an object".to_string())?;
        let name = obj
            .get("name")
            .or_else(|| obj.get("source_set"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing name/source_set in pinned source_sets record".to_string())?
            .to_string();
        let paths = obj
            .get("paths")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                "missing or invalid paths array in pinned source_sets record".to_string()
            })?
            .iter()
            .map(|p| {
                p.as_str()
                    .map(String::from)
                    .ok_or_else(|| "non-string in paths array".to_string())
            })
            .collect::<Result<Vec<String>, _>>()?;
        let identity_count = obj
            .get("identity_count")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                "missing or invalid identity_count in pinned source_sets record".to_string()
            })? as usize;
        let sha256_digest = obj
            .get("sha256_digest")
            .or_else(|| obj.get("digest"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing sha256_digest in pinned source_sets record".to_string())?
            .to_string();

        records.push(PinnedSourceSetRecord {
            name,
            paths,
            identity_count,
            sha256_digest,
        });
    }

    Ok(records)
}

fn compute_length_framed_set_digest(
    repo_root: &std::path::Path,
    paths: &[String],
) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for p in paths {
        let abs = repo_root.join(p);
        let bytes = if abs.is_file() {
            std::fs::read(&abs).map_err(|e| format!("read {p}: {e}"))?
        } else {
            b"<missing>".to_vec()
        };
        hasher.update(&(p.len() as u64).to_be_bytes());
        hasher.update(p.as_bytes());
        hasher.update(&(bytes.len() as u64).to_be_bytes());
        hasher.update(&bytes);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn compare_pinned_and_discovered_source_sets(
    pinned: &[PinnedSourceSetRecord],
    discovered: &BTreeMap<String, DiscoveredSourceSet>,
) -> Result<(), String> {
    let required_names: BTreeSet<String> = [
        "ordinary_parity_corpus",
        "tier1_ec_cases",
        "tier1_gate_denominators",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let mut pinned_names = BTreeSet::new();
    for p in pinned {
        if !required_names.contains(&p.name) {
            return Err(format!("unknown pinned source_set name '{}'", p.name));
        }
        if !pinned_names.insert(p.name.clone()) {
            return Err(format!("duplicate pinned source_set name '{}'", p.name));
        }
    }

    if pinned_names != required_names {
        return Err(format!(
            "pinned source_sets names {:?} do not match exact required set {:?}",
            pinned_names, required_names
        ));
    }

    let disc_names: BTreeSet<String> = discovered.keys().cloned().collect();
    if disc_names != required_names {
        return Err(format!(
            "discovered source_sets names {:?} do not match exact required set {:?}",
            disc_names, required_names
        ));
    }

    for p in pinned {
        let disc = discovered.get(&p.name).ok_or_else(|| {
            format!(
                "pinned source_set '{}' not found in discovered source_sets",
                p.name
            )
        })?;

        if p.paths != disc.paths {
            return Err(format!(
                "pinned paths for source_set '{}' do not match discovered paths",
                p.name
            ));
        }

        if p.identity_count != disc.identity_count {
            return Err(format!(
                "pinned identity_count {} for source_set '{}' != discovered identity_count {}",
                p.identity_count, p.name, disc.identity_count
            ));
        }

        if p.sha256_digest != disc.sha256_digest {
            return Err(format!(
                "pinned sha256_digest '{}' for source_set '{}' != discovered sha256_digest '{}'",
                p.sha256_digest, p.name, disc.sha256_digest
            ));
        }
    }

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OutOfScopeDisposition {
    source_set: String,
    source_identity: String,
    reason: String,
    reviewed_against: String,
}

fn validate_denominator_manifest(
    manifest_toml: &toml::Value,
    dir_name: &str,
    parsed_identities_len: usize,
    actual_sha256: &str,
) -> Result<(), String> {
    let family = manifest_toml
        .get("family")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("manifest in '{dir_name}' missing required field 'family'"))?;
    if family != dir_name {
        return Err(format!(
            "manifest family '{family}' does not match directory name '{dir_name}'"
        ));
    }

    let cap = manifest_toml
        .get("capability")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("manifest in '{dir_name}' missing required field 'capability'"))?;
    if cap != "mamba-core-semantics" {
        return Err(format!(
            "manifest capability '{cap}' is not 'mamba-core-semantics' in '{dir_name}'"
        ));
    }

    let manifest_row_count = manifest_toml
        .get("row_count")
        .and_then(|v| v.as_integer())
        .ok_or_else(|| format!("manifest in '{dir_name}' missing required field 'row_count'"))?
        as usize;

    let expected_sha256 = manifest_toml
        .get("denominator_sha256")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!("manifest in '{dir_name}' missing required field 'denominator_sha256'")
        })?;

    if actual_sha256 != expected_sha256 {
        return Err(format!(
            "denominator sha256 mismatch in '{dir_name}': expected {expected_sha256}, found {actual_sha256}"
        ));
    }

    if parsed_identities_len != manifest_row_count {
        return Err(format!(
            "row_count {manifest_row_count} in manifest '{dir_name}' does not match parsed identities count {parsed_identities_len}"
        ));
    }

    Ok(())
}

fn discover_authoritative_source_sets(
    repo_root: &std::path::Path,
) -> Result<BTreeMap<String, DiscoveredSourceSet>, String> {
    use sha2::{Digest, Sha256};
    let mut sets = BTreeMap::new();

    // 1. ordinary_parity_corpus
    let mut ordinary_manifest_paths = Vec::new();
    let mut ordinary_paths = Vec::new();
    let mut ordinary_identities = BTreeSet::new();

    let manifests_dir = repo_root.join("projects/mamba/tests/harness/cpython/config/manifests");
    fn collect_manifest_tomls(
        dir: &std::path::Path,
        repo_root: &std::path::Path,
        paths: &mut Vec<String>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_manifest_tomls(&path, repo_root, paths);
                } else if path.extension() == Some(OsStr::new("toml")) {
                    if let Ok(rel) = path.strip_prefix(repo_root) {
                        paths.push(rel.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    collect_manifest_tomls(&manifests_dir, repo_root, &mut ordinary_manifest_paths);
    ordinary_manifest_paths.sort();

    for path_str in &ordinary_manifest_paths {
        ordinary_paths.push(path_str.clone());
        let abs_path = repo_root.join(path_str);
        let raw_toml = std::fs::read_to_string(&abs_path)
            .map_err(|e| format!("read ordinary manifest '{path_str}': {e}"))?;
        let toml_val: toml::Value = raw_toml
            .parse()
            .map_err(|e| format!("parse TOML in ordinary manifest '{path_str}': {e}"))?;

        let top_bucket = toml_val.get("bucket").and_then(|v| v.as_str());
        let top_lib = toml_val.get("lib").and_then(|v| v.as_str());

        let cases = toml_val
            .get("case")
            .or_else(|| toml_val.get("cases"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("manifest '{path_str}' missing 'case' array"))?;

        if cases.is_empty() {
            return Err(format!("manifest '{path_str}' has empty 'case' array"));
        }

        for c in cases {
            let case_obj = c
                .as_table()
                .ok_or_else(|| format!("item in 'case' array of '{path_str}' is not a table"))?;

            let case_name = case_obj
                .get("case")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| {
                    format!("case in '{path_str}' missing or empty string field 'case'")
                })?;

            let source_id = format!("{path_str}#case={case_name}");
            parse_exact_manifest_case_identity(&source_id).map_err(|e| {
                format!("discovered manifest identity invalid in '{path_str}': {e}")
            })?;
            if !ordinary_identities.insert(source_id.clone()) {
                return Err(format!(
                    "duplicate manifest case identity '{source_id}' in manifest '{path_str}'"
                ));
            }
        }
    }

    let gaps_path = repo_root.join("projects/mamba/tests/harness/cpython/config/behavior_gaps.txt");
    if gaps_path.is_file() {
        if let Ok(rel) = gaps_path.strip_prefix(repo_root) {
            ordinary_paths.push(rel.to_string_lossy().to_string());
        }
        let gaps_raw = std::fs::read_to_string(&gaps_path)
            .map_err(|e| format!("read behavior_gaps.txt: {e}"))?;
        for line in gaps_raw.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                ordinary_identities.insert(trimmed.to_string());
            }
        }
    }

    let div_path =
        repo_root.join("projects/mamba/tests/harness/cpython/config/type_divergences.txt");
    if div_path.is_file() {
        if let Ok(rel) = div_path.strip_prefix(repo_root) {
            ordinary_paths.push(rel.to_string_lossy().to_string());
        }
        let div_raw = std::fs::read_to_string(&div_path)
            .map_err(|e| format!("read type_divergences.txt: {e}"))?;
        for line in div_raw.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                ordinary_identities.insert(trimmed.to_string());
            }
        }
    }

    ordinary_paths.sort();
    ordinary_paths.dedup();

    let ordinary_digest = compute_length_framed_set_digest(repo_root, &ordinary_paths)?;

    sets.insert(
        "ordinary_parity_corpus".to_string(),
        DiscoveredSourceSet {
            name: "ordinary_parity_corpus".to_string(),
            paths: ordinary_paths,
            identity_count: ordinary_identities.len(),
            sha256_digest: ordinary_digest,
            source_identities: ordinary_identities.into_iter().collect(),
        },
    );

    // 2. tier1_ec_cases
    let mut ec_paths = Vec::new();
    let mut ec_identities = BTreeSet::new();

    let ec_dirs = [
        repo_root.join("projects/mamba/external-contracts/behavior"),
        repo_root.join("projects/mamba/external-contracts/stability"),
        repo_root.join("projects/mamba/external-contracts/efficiency"),
    ];

    for dir in &ec_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension() == Some(OsStr::new("md")) {
                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| format!("read {}: {e}", path.display()))?;
                    let lines: Vec<&str> = content.lines().collect();
                    let mut in_yaml = false;
                    let mut yaml_lines = Vec::new();
                    let mut found_cap = false;
                    for line in lines {
                        let trimmed = line.trim();
                        if trimmed.starts_with("```yaml") {
                            in_yaml = true;
                            yaml_lines.clear();
                        } else if in_yaml && trimmed.starts_with("```") {
                            in_yaml = false;
                            let block = yaml_lines.join("\n");
                            if block.contains("e2e_tests:") {
                                let mut current_id = String::new();
                                let mut current_cap = String::new();
                                for yline in block.lines() {
                                    let ytrim = yline.trim();
                                    if ytrim.starts_with("- id:") || ytrim.starts_with("id:") {
                                        if !current_id.is_empty()
                                            && current_cap == "mamba-core-semantics"
                                        {
                                            ec_identities.insert(current_id.clone());
                                            found_cap = true;
                                        }
                                        current_id = ytrim
                                            .split(':')
                                            .nth(1)
                                            .unwrap_or("")
                                            .trim()
                                            .trim_matches('"')
                                            .trim_matches('\'')
                                            .to_string();
                                        current_cap.clear();
                                    } else if ytrim.starts_with("capability_id:") {
                                        current_cap = ytrim
                                            .split(':')
                                            .nth(1)
                                            .unwrap_or("")
                                            .trim()
                                            .trim_matches('"')
                                            .trim_matches('\'')
                                            .to_string();
                                    }
                                }
                                if !current_id.is_empty() && current_cap == "mamba-core-semantics" {
                                    ec_identities.insert(current_id.clone());
                                    found_cap = true;
                                }
                            }
                        } else if in_yaml {
                            yaml_lines.push(line);
                        }
                    }
                    if found_cap {
                        if let Ok(rel) = path.strip_prefix(repo_root) {
                            ec_paths.push(rel.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    ec_paths.sort();
    ec_paths.dedup();

    let ec_digest = compute_length_framed_set_digest(repo_root, &ec_paths)?;

    sets.insert(
        "tier1_ec_cases".to_string(),
        DiscoveredSourceSet {
            name: "tier1_ec_cases".to_string(),
            paths: ec_paths,
            identity_count: ec_identities.len(),
            sha256_digest: ec_digest,
            source_identities: ec_identities.into_iter().collect(),
        },
    );

    // 3. tier1_gate_denominators
    let mut denom_paths = Vec::new();
    let mut denom_identities = BTreeSet::new();

    let gates_dir = repo_root.join("projects/mamba/tests/governance/gates");
    if let Ok(entries) = std::fs::read_dir(&gates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                    if dir_name.starts_with("t1_") && dir_name.ends_with("_denominator") {
                        let manifest_path = path.join("manifest.toml");
                        let denom_txt = path.join("denominator.txt");
                        let manifest_rel = format!(
                            "projects/mamba/tests/governance/gates/{dir_name}/manifest.toml"
                        );
                        let denom_rel = format!(
                            "projects/mamba/tests/governance/gates/{dir_name}/denominator.txt"
                        );

                        denom_paths.push(manifest_rel);
                        denom_paths.push(denom_rel.clone());

                        if manifest_path.is_file() {
                            let raw_manifest = std::fs::read_to_string(&manifest_path)
                                .map_err(|e| format!("read {}: {e}", manifest_path.display()))?;
                            let manifest_toml: toml::Value = raw_manifest.parse().map_err(|e| {
                                format!("parse TOML in {}: {e}", manifest_path.display())
                            })?;

                            if !denom_txt.is_file() {
                                return Err(format!(
                                    "denominator gate directory '{}' has manifest.toml but is missing denominator.txt",
                                    path.display()
                                ));
                            }

                            let denom_bytes = std::fs::read(&denom_txt)
                                .map_err(|e| format!("read {}: {e}", denom_txt.display()))?;
                            let actual_sha = format!("{:x}", Sha256::digest(&denom_bytes));

                            let denom_raw = String::from_utf8_lossy(&denom_bytes);
                            let parsed_identities: Vec<String> = denom_raw
                                .lines()
                                .map(|l| l.trim())
                                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                                .map(|l| l.to_string())
                                .collect();

                            validate_denominator_manifest(
                                &manifest_toml,
                                dir_name,
                                parsed_identities.len(),
                                &actual_sha,
                            )?;

                            if parsed_identities.is_empty() {
                                denom_identities.insert(denom_rel);
                            } else {
                                for id in parsed_identities {
                                    denom_identities.insert(id);
                                }
                            }
                        } else {
                            // Missing manifest.toml: represent missing/empty state deterministically
                            if denom_txt.is_file() {
                                let denom_bytes = std::fs::read(&denom_txt)
                                    .map_err(|e| format!("read {}: {e}", denom_txt.display()))?;
                                let denom_raw = String::from_utf8_lossy(&denom_bytes);
                                let parsed_identities: Vec<String> = denom_raw
                                    .lines()
                                    .map(|l| l.trim())
                                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                                    .map(|l| l.to_string())
                                    .collect();
                                if parsed_identities.is_empty() {
                                    denom_identities.insert(denom_rel);
                                } else {
                                    for id in parsed_identities {
                                        denom_identities.insert(id);
                                    }
                                }
                            } else {
                                denom_identities.insert(denom_rel);
                            }
                        }
                    }
                }
            }
        }
    }

    denom_paths.sort();
    denom_paths.dedup();

    let denom_digest = compute_length_framed_set_digest(repo_root, &denom_paths)?;

    sets.insert(
        "tier1_gate_denominators".to_string(),
        DiscoveredSourceSet {
            name: "tier1_gate_denominators".to_string(),
            paths: denom_paths,
            identity_count: denom_identities.len(),
            sha256_digest: denom_digest,
            source_identities: denom_identities.into_iter().collect(),
        },
    );

    Ok(sets)
}

fn reconcile_source_sets_and_inventory(
    discovered_sets: &BTreeMap<String, DiscoveredSourceSet>,
    lock_dispositions: &[OutOfScopeDisposition],
    rows: &[RawCaseRow],
) -> Result<(), String> {
    for disp in lock_dispositions {
        if disp.source_set.trim().is_empty() {
            return Err(format!(
                "out_of_scope disposition for '{}' missing source_set qualification",
                disp.source_identity
            ));
        }
        if disp.reason.trim().is_empty() {
            return Err(format!(
                "out_of_scope disposition for '({}:{})' missing reviewer-auditable reason",
                disp.source_set, disp.source_identity
            ));
        }
        if disp.reviewed_against.trim().is_empty() {
            return Err(format!(
                "out_of_scope disposition for '({}:{})' missing reviewed_against issue",
                disp.source_set, disp.source_identity
            ));
        }
    }

    let out_of_scope_pairs: BTreeSet<(&str, &str)> = lock_dispositions
        .iter()
        .map(|d| (d.source_set.as_str(), d.source_identity.as_str()))
        .collect();

    let mut row_set_identities = BTreeSet::new();
    let mut probe_ids = BTreeSet::new();

    for row in rows {
        if !probe_ids.insert(&row.probe_id) {
            return Err(format!(
                "duplicate probe_id detected across cases.jsonl rows: '{}'",
                row.probe_id
            ));
        }

        if !discovered_sets.contains_key(&row.source_set) {
            return Err(format!(
                "row '{}' has unknown source_set '{}'",
                row.case_id, row.source_set
            ));
        }

        row_set_identities.insert((row.source_set.as_str(), row.source_identity.as_str()));
    }

    for (set_name, set_info) in discovered_sets {
        for src_id in &set_info.source_identities {
            let in_rows = row_set_identities.contains(&(set_name.as_str(), src_id.as_str()));
            let in_out_of_scope =
                out_of_scope_pairs.contains(&(set_name.as_str(), src_id.as_str()));

            if !in_rows && !in_out_of_scope {
                return Err(format!(
                    "discovered source_identity '{}' in set '{}' is neither in cases.jsonl nor covered by an out_of_scope disposition for that set",
                    src_id, set_name
                ));
            }
        }
    }

    for (set_name, src_id) in &row_set_identities {
        let is_discovered = discovered_sets.get(*set_name).map_or(false, |s| {
            s.source_identities.iter().any(|id| id.as_str() == *src_id)
        });
        if !is_discovered {
            return Err(format!(
                "cases.jsonl row refers to undiscovered source_identity '{}' in set '{}'",
                src_id, set_name
            ));
        }
    }

    for disp in lock_dispositions {
        let is_discovered = discovered_sets.get(&disp.source_set).map_or(false, |s| {
            s.source_identities
                .iter()
                .any(|id| id.as_str() == disp.source_identity.as_str())
        });
        if !is_discovered {
            return Err(format!(
                "out_of_scope disposition refers to undiscovered source_identity '{}' in set '{}'",
                disp.source_identity, disp.source_set
            ));
        }
    }

    Ok(())
}

const ALLOWED_POST_PIN_ARTIFACT_PATHS: &[&str] = &[
    "projects/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/cases.jsonl",
    "projects/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/manifest.toml",
    "projects/mamba/external-contracts/evidence/mamba-t1-oracle-hierarchy-lock.json",
];

fn validate_post_pin_changed_paths<S: AsRef<str>>(changed_paths: &[S]) -> Result<(), String> {
    for path in changed_paths {
        let p = path.as_ref();
        if !ALLOWED_POST_PIN_ARTIFACT_PATHS.contains(&p) {
            return Err(format!(
                "unauthorized post-pin repository modification: '{p}' is not an allowed post-pin artifact"
            ));
        }
    }
    Ok(())
}

fn validate_source_revision_pin(pin: &str, live_head: &str) -> Result<(), String> {
    if pin.len() != 40 || !pin.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
        return Err(format!(
            "manifest mamba_git_sha must be exactly 40 lowercase hex characters, got '{pin}'"
        ));
    }

    let cat_file_status = Command::new("git")
        .args(["cat-file", "-e", &format!("{pin}^{{commit}}")])
        .status()
        .map_err(|e| format!("failed to execute git cat-file: {e}"))?;
    if !cat_file_status.success() {
        return Err(format!("pinned git SHA '{pin}' is not a valid commit"));
    }

    let is_ancestor_status = Command::new("git")
        .args(["merge-base", "--is-ancestor", pin, live_head])
        .status()
        .map_err(|e| format!("failed to execute git merge-base: {e}"))?;
    if !is_ancestor_status.success() {
        return Err(format!(
            "pinned git SHA '{pin}' is not live HEAD or an ancestor of live HEAD '{live_head}'"
        ));
    }

    let diff_output = Command::new("git")
        .args(["diff", "--name-only", &format!("{pin}..{live_head}"), "--"])
        .output()
        .map_err(|e| format!("failed to execute git diff: {e}"))?;
    if !diff_output.status.success() {
        return Err(format!(
            "git diff failed between '{pin}' and '{live_head}': {:?}",
            diff_output.status
        ));
    }
    let stdout = String::from_utf8(diff_output.stdout)
        .map_err(|e| format!("git diff output is not UTF-8: {e}"))?;
    let changed_paths: Vec<&str> = stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    validate_post_pin_changed_paths(&changed_paths)?;

    Ok(())
}

fn run_oracle_hierarchy_in_test_mutation_canaries() {
    // Pure helper canaries for allowed post-pin artifact paths
    assert!(
        validate_post_pin_changed_paths::<&str>(&[]).is_ok(),
        "Post-pin canary (empty path list accepts) FAILED"
    );
    assert!(
        validate_post_pin_changed_paths(&[
            "projects/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/cases.jsonl",
            "projects/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/manifest.toml",
            "projects/mamba/external-contracts/evidence/mamba-t1-oracle-hierarchy-lock.json",
        ])
        .is_ok(),
        "Post-pin canary (three allowed paths accept) FAILED"
    );
    assert!(
        validate_post_pin_changed_paths(&[
            "projects/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/cases.jsonl.bak",
        ])
        .is_err(),
        "Post-pin canary (near-miss filename rejects) FAILED"
    );
    assert!(
        validate_post_pin_changed_paths(&["projects/mamba/src/lib.rs",]).is_err(),
        "Post-pin canary (Mamba source path rejects) FAILED"
    );
    let mamba_exe = mamba_bin();
    let mamba_bytes = std::fs::read(&mamba_exe).expect("read mamba bin for canary");
    use sha2::{Digest, Sha256};
    let dummy_mamba_sha = format!("{:x}", Sha256::digest(&mamba_bytes));
    let dummy_git_head = "1111111111111111111111111111111111111111";
    let dummy_platform = "aarch64-apple-darwin";

    let dummy_row = RawCaseRow {
        case_id: "dummy_case".to_string(),
        channel: "behavior".to_string(),
        tier1_dimension: "behavior".to_string(),
        oracle_kind: "cpython312_identity".to_string(),
        oracle_executable: "/usr/bin/python3.12".to_string(),
        oracle_version: "Python 3.12".to_string(),
        oracle_command: "python3.12 dummy.py".to_string(),
        sut_command: format!(
            "{} run projects/mamba/external-contracts/behavior/mamba-t1-to-thread-gather-results.md",
            mamba_exe.to_string_lossy()
        ),
        expected_divergence_class: "none".to_string(),
        expected_outcome_kind: "ok".to_string(),
        expected_probe_anchor: "ANCHOR".to_string(),
        expected_result_channel: "behavior".to_string(),
        expected_terminal_classification: "green".to_string(),
        diagnostic_class: None,
        diagnostic_span: None,
        fixture_or_probe_path: "projects/mamba/external-contracts/behavior/mamba-t1-to-thread-gather-results.md".to_string(),
        mamba_binary_sha256: dummy_mamba_sha.clone(),
        mamba_git_sha: "1111111111111111111111111111111111111111".to_string(),
        platform: "aarch64-apple-darwin".to_string(),
        source_set: "tier1_ec_cases".to_string(),
        source_identity: "mamba-t1-to-thread-gather-results".to_string(),
        probe_id: "dummy_probe".to_string(),
        sample_role: "green".to_string(),
    };

    // 1. Unknown route canary
    let mut bad_route = dummy_row.clone();
    bad_route.oracle_kind = "unknown_route".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_route,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 1 (unknown route) FAILED"
    );

    // 2. Unknown version canary
    let mut bad_ver = dummy_row.clone();
    bad_ver.oracle_version = "Python 2.7".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_ver,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 2 (unknown version) FAILED"
    );

    // 3. Unknown class canary
    let mut bad_class = dummy_row.clone();
    bad_class.expected_divergence_class = "unknown_class".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_class,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 3 (unknown class) FAILED"
    );

    // 4. Self-oracle canary
    let mut bad_self = dummy_row.clone();
    bad_self.oracle_command = bad_self.sut_command.clone();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_self,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 4 (self-oracle) FAILED"
    );

    // 5. Synthetic print-only SUT command canary
    let mut bad_synthetic = dummy_row.clone();
    bad_synthetic.sut_command = "python -c \"print('ANCHOR')\"".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_synthetic,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 5 (synthetic print-only SUT command) FAILED"
    );

    // 6. Non-Mamba SUT command canary
    let mut bad_non_mamba = dummy_row.clone();
    bad_non_mamba.sut_command = "python3 run_fixture.py".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_non_mamba,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 6 (non-Mamba SUT command) FAILED"
    );

    // 7. Unbound SUT command fixture canary
    let mut bad_unbound_sut = dummy_row.clone();
    bad_unbound_sut.sut_command = format!("{} run other_file.py", mamba_exe.to_string_lossy());
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_unbound_sut,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 7 (unbound SUT command fixture) FAILED"
    );

    // 8. Missing diagnostic_class for compile_error canary
    let mut bad_diag_cls = dummy_row.clone();
    bad_diag_cls.expected_outcome_kind = "compile_error".to_string();
    bad_diag_cls.expected_divergence_class = "force_typed_compile_reject".to_string();
    bad_diag_cls.diagnostic_class = None;
    bad_diag_cls.diagnostic_span = Some("foo.py:1:1".to_string());
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_diag_cls,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 8 (missing diagnostic_class for compile_error) FAILED"
    );

    // 9. Missing diagnostic_span for compile_error canary
    let mut bad_diag_span = dummy_row.clone();
    bad_diag_span.expected_outcome_kind = "compile_error".to_string();
    bad_diag_span.expected_divergence_class = "force_typed_compile_reject".to_string();
    bad_diag_span.diagnostic_class = Some("TypeMismatch".to_string());
    bad_diag_span.diagnostic_span = None;
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_diag_span,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 9 (missing diagnostic_span for compile_error) FAILED"
    );

    // 10. Non-existent fixture file canary
    let mut bad_fixture_path = dummy_row.clone();
    bad_fixture_path.fixture_or_probe_path = "non_existent_file_path_xyz.py".to_string();
    bad_fixture_path.sut_command = format!(
        "{} run non_existent_file_path_xyz.py",
        mamba_exe.to_string_lossy()
    );
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_fixture_path,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 10 (non-existent fixture file) FAILED"
    );

    // 11. Zero execution canary
    assert!(
        reconcile_observed_dataset(&[], 0, "sha").is_err(),
        "Canary 11 (zero execution) FAILED"
    );

    // 12. Channel mismatch canary
    let mut bad_ch = dummy_row.clone();
    bad_ch.expected_result_channel = "compile".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_ch,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 12 (channel mismatch) FAILED"
    );

    // 13. Missing identity canary
    let mut bad_id = dummy_row.clone();
    bad_id.mamba_binary_sha256 =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_id,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 13 (missing identity) FAILED"
    );

    // 14. Wrong anchor canary
    let dummy_ev_ok = ExecutedCommandEvidence {
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        stdout_sha256: "abc".to_string(),
        stderr_sha256: "def".to_string(),
        stdout_text: "output without anchor".to_string(),
        stderr_text: "".to_string(),
    };
    assert!(
        classify_and_reconcile_row(&dummy_row, &dummy_ev_ok, &dummy_ev_ok).is_err(),
        "Canary 14 (wrong anchor) FAILED"
    );

    // 15. Unclassified terminal state canary
    let mut bad_unclass = dummy_row.clone();
    bad_unclass.expected_outcome_kind = "bogus".to_string();
    assert!(
        classify_and_reconcile_row(&bad_unclass, &dummy_ev_ok, &dummy_ev_ok).is_err(),
        "Canary 15 (unclassified state) FAILED"
    );

    // 16. Property red exit 0 canary
    let mut bad_prop_red = dummy_row.clone();
    bad_prop_red.expected_outcome_kind = "property_red".to_string();
    bad_prop_red.expected_terminal_classification = "intentional_red".to_string();
    let dummy_ev_anchor_ok = ExecutedCommandEvidence {
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        stdout_sha256: "abc".to_string(),
        stderr_sha256: "def".to_string(),
        stdout_text: "ANCHOR".to_string(),
        stderr_text: "".to_string(),
    };
    assert!(
        classify_and_reconcile_row(&bad_prop_red, &dummy_ev_anchor_ok, &dummy_ev_anchor_ok)
            .is_err(),
        "Canary 16 (property red exit 0) FAILED"
    );

    // 17. Inventory SHA-256 identity mismatch canary
    let dummy_record = ObservedRowRecord {
        case_id: dummy_row.case_id.clone(),
        channel: dummy_row.channel.clone(),
        tier1_dimension: dummy_row.tier1_dimension.clone(),
        derived_classification: "green".to_string(),
        mamba_binary_sha256: dummy_mamba_sha.clone(),
        mamba_git_sha: dummy_git_head.to_string(),
        platform: dummy_platform.to_string(),
        oracle_executable: dummy_row.oracle_executable.clone(),
        oracle_version: dummy_row.oracle_version.clone(),
        inventory_sha256: "bad_inventory_sha".to_string(),
        oracle_ev: dummy_ev_ok.clone(),
        sut_ev: dummy_ev_ok.clone(),
    };
    assert!(
        reconcile_observed_dataset(&[dummy_record], 1, "expected_inventory_sha").is_err(),
        "Canary 17 (inventory sha mismatch) FAILED"
    );

    // 18. Fake /mamba SUT executable canary
    let mut bad_fake_mamba = dummy_row.clone();
    bad_fake_mamba.sut_command =
        "/tmp/fake/mamba run projects/mamba/external-contracts/behavior/mamba-t1-to-thread-gather-results.md".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_fake_mamba,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 18 (fake /mamba executable) FAILED"
    );

    // 19. Fixture path only in comment canary
    let mut bad_comment_fixture = dummy_row.clone();
    bad_comment_fixture.sut_command = format!(
        "{} run other_file.py # projects/mamba/external-contracts/behavior/mamba-t1-to-thread-gather-results.md",
        mamba_exe.to_string_lossy()
    );
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_comment_fixture,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 19 (fixture only in comment) FAILED"
    );

    // 20. Fixture path only as substring canary
    let mut bad_substring_fixture = dummy_row.clone();
    bad_substring_fixture.sut_command = format!(
        "{} run projects/mamba/external-contracts/behavior/mamba-t1-to-thread-gather-results.md_extra",
        mamba_exe.to_string_lossy()
    );
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_substring_fixture,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 20 (fixture only as substring) FAILED"
    );

    // 21. Unknown source_set canary
    let mut bad_src_set = dummy_row.clone();
    bad_src_set.source_set = "unknown_set".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_src_set,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 21 (unknown source_set) FAILED"
    );

    // 22. Unknown sample_role canary
    let mut bad_role = dummy_row.clone();
    bad_role.sample_role = "unknown_role".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_role,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 22 (unknown sample_role) FAILED"
    );

    // 23. Empty probe_id canary
    let mut bad_probe_id = dummy_row.clone();
    bad_probe_id.probe_id = "".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_probe_id,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 23 (empty probe_id) FAILED"
    );

    // 24. Empty source_identity canary
    let mut bad_src_id = dummy_row.clone();
    bad_src_id.source_identity = "".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_src_id,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 24 (empty source_identity) FAILED"
    );

    // 25. Duplicate probe_id canary
    let mut dup_row1 = dummy_row.clone();
    let mut dup_row2 = dummy_row.clone();
    dup_row1.case_id = "case1".to_string();
    dup_row2.case_id = "case2".to_string();
    dup_row1.probe_id = "same_probe_id".to_string();
    dup_row2.probe_id = "same_probe_id".to_string();
    let empty_sets = BTreeMap::new();
    assert!(
        reconcile_source_sets_and_inventory(&empty_sets, &[], &[dup_row1, dup_row2]).is_err(),
        "Canary 25 (duplicate probe_id) FAILED"
    );

    // 26. Omitted authoritative identity mutation canary
    let mut mock_sets_omitted = BTreeMap::new();
    mock_sets_omitted.insert(
        "tier1_ec_cases".to_string(),
        DiscoveredSourceSet {
            name: "tier1_ec_cases".to_string(),
            paths: vec!["p1".to_string()],
            identity_count: 2,
            sha256_digest: "digest1".to_string(),
            source_identities: vec![
                "mamba-t1-to-thread-gather-results".to_string(),
                "missing_authoritative_id_xyz".to_string(),
            ],
        },
    );
    assert!(
        reconcile_source_sets_and_inventory(&mock_sets_omitted, &[], &[dummy_row.clone()]).is_err(),
        "Canary 26 (omitted authoritative identity mutation) FAILED"
    );

    // 27. Stale row identity mutation canary
    let mut mock_sets_stale = BTreeMap::new();
    mock_sets_stale.insert(
        "tier1_ec_cases".to_string(),
        DiscoveredSourceSet {
            name: "tier1_ec_cases".to_string(),
            paths: vec!["p1".to_string()],
            identity_count: 1,
            sha256_digest: "digest1".to_string(),
            source_identities: vec!["mamba-t1-to-thread-gather-results".to_string()],
        },
    );
    let mut stale_row = dummy_row.clone();
    stale_row.source_identity = "stale_row_identity_xyz".to_string();
    assert!(
        reconcile_source_sets_and_inventory(&mock_sets_stale, &[], &[stale_row]).is_err(),
        "Canary 27 (stale row identity mutation) FAILED"
    );

    // 28. Blank disposition reason / reviewer / source_set mutation canary
    let bad_disp_blank_reason = OutOfScopeDisposition {
        source_set: "tier1_gate_denominators".to_string(),
        source_identity: "mamba-t1-to-thread-gather-results".to_string(),
        reason: "".to_string(),
        reviewed_against: "#2022".to_string(),
    };
    assert!(
        reconcile_source_sets_and_inventory(&mock_sets_stale, &[bad_disp_blank_reason], &[])
            .is_err(),
        "Canary 28 (blank disposition reason mutation) FAILED"
    );

    let bad_disp_blank_rev = OutOfScopeDisposition {
        source_set: "tier1_gate_denominators".to_string(),
        source_identity: "mamba-t1-to-thread-gather-results".to_string(),
        reason: "valid reason".to_string(),
        reviewed_against: "".to_string(),
    };
    assert!(
        reconcile_source_sets_and_inventory(&mock_sets_stale, &[bad_disp_blank_rev], &[]).is_err(),
        "Canary 28b (blank disposition reviewer mutation) FAILED"
    );

    let bad_disp_blank_set = OutOfScopeDisposition {
        source_set: "".to_string(),
        source_identity: "mamba-t1-to-thread-gather-results".to_string(),
        reason: "valid reason".to_string(),
        reviewed_against: "#2022".to_string(),
    };
    assert!(
        reconcile_source_sets_and_inventory(&mock_sets_stale, &[bad_disp_blank_set], &[]).is_err(),
        "Canary 28c (blank disposition source_set mutation) FAILED"
    );

    // 29. Changed source digest mutation canary exercising compare_pinned_and_discovered_source_sets
    let pinned_rec1 = PinnedSourceSetRecord {
        name: "ordinary_parity_corpus".to_string(),
        paths: vec!["p1".to_string()],
        identity_count: 1,
        sha256_digest: "digest1".to_string(),
    };
    let pinned_rec2 = PinnedSourceSetRecord {
        name: "tier1_ec_cases".to_string(),
        paths: vec!["p2".to_string()],
        identity_count: 1,
        sha256_digest: "digest_pinned_1111".to_string(),
    };
    let pinned_rec3 = PinnedSourceSetRecord {
        name: "tier1_gate_denominators".to_string(),
        paths: vec!["p3".to_string()],
        identity_count: 1,
        sha256_digest: "digest3".to_string(),
    };
    let mut mock_discovered_sets = BTreeMap::new();
    mock_discovered_sets.insert(
        "ordinary_parity_corpus".to_string(),
        DiscoveredSourceSet {
            name: "ordinary_parity_corpus".to_string(),
            paths: vec!["p1".to_string()],
            identity_count: 1,
            sha256_digest: "digest1".to_string(),
            source_identities: vec!["id1".to_string()],
        },
    );
    mock_discovered_sets.insert(
        "tier1_ec_cases".to_string(),
        DiscoveredSourceSet {
            name: "tier1_ec_cases".to_string(),
            paths: vec!["p2".to_string()],
            identity_count: 1,
            sha256_digest: "digest_mutated_9999".to_string(),
            source_identities: vec!["mamba-t1-to-thread-gather-results".to_string()],
        },
    );
    mock_discovered_sets.insert(
        "tier1_gate_denominators".to_string(),
        DiscoveredSourceSet {
            name: "tier1_gate_denominators".to_string(),
            paths: vec!["p3".to_string()],
            identity_count: 1,
            sha256_digest: "digest3".to_string(),
            source_identities: vec!["id3".to_string()],
        },
    );
    assert!(
        compare_pinned_and_discovered_source_sets(
            &[
                pinned_rec1.clone(),
                pinned_rec2.clone(),
                pinned_rec3.clone()
            ],
            &mock_discovered_sets
        )
        .is_err(),
        "Canary 29 (changed source digest mutation) FAILED"
    );

    // 30. Undiscovered disposition identity mutation canary
    let bad_disp_undiscovered = OutOfScopeDisposition {
        source_set: "tier1_ec_cases".to_string(),
        source_identity: "nonexistent_disposition_id_xyz".to_string(),
        reason: "valid reason".to_string(),
        reviewed_against: "#2022".to_string(),
    };
    assert!(
        reconcile_source_sets_and_inventory(
            &mock_sets_stale,
            &[bad_disp_undiscovered],
            &[dummy_row.clone()]
        )
        .is_err(),
        "Canary 30 (undiscovered disposition identity mutation) FAILED"
    );

    // 31. Duplicate pinned source-set name canary
    assert!(
        compare_pinned_and_discovered_source_sets(
            &[
                pinned_rec1.clone(),
                pinned_rec1.clone(),
                pinned_rec3.clone()
            ],
            &mock_discovered_sets
        )
        .is_err(),
        "Canary 31 (duplicate pinned source-set name) FAILED"
    );

    // 32. Unknown pinned source-set name canary
    let unknown_pinned_rec = PinnedSourceSetRecord {
        name: "unknown_source_set_xyz".to_string(),
        paths: vec!["p2".to_string()],
        identity_count: 1,
        sha256_digest: "digest1".to_string(),
    };
    assert!(
        compare_pinned_and_discovered_source_sets(
            &[unknown_pinned_rec, pinned_rec2, pinned_rec3],
            &mock_discovered_sets
        )
        .is_err(),
        "Canary 32 (unknown pinned source-set name) FAILED"
    );

    // 33. Denominator row-count mismatch canary
    let denom_toml_mismatch: toml::Value = toml::from_str(
        r#"
family = "t1_test_denominator"
capability = "mamba-core-semantics"
row_count = 10
denominator_sha256 = "abc"
"#,
    )
    .unwrap();
    assert!(
        validate_denominator_manifest(&denom_toml_mismatch, "t1_test_denominator", 5, "abc")
            .is_err(),
        "Canary 33 (denominator row-count mismatch) FAILED"
    );

    // 34. Missing-manifest disposition canary
    let mock_missing_manifest_set = DiscoveredSourceSet {
        name: "tier1_gate_denominators".to_string(),
        paths: vec![
            "projects/mamba/tests/governance/gates/t1_multicore_scaling_denominator/denominator.txt".to_string(),
        ],
        identity_count: 1,
        sha256_digest: "digest_missing".to_string(),
        source_identities: vec![
            "projects/mamba/tests/governance/gates/t1_multicore_scaling_denominator/denominator.txt".to_string(),
        ],
    };
    let mut mock_discovered_missing = BTreeMap::new();
    mock_discovered_missing.insert(
        "tier1_gate_denominators".to_string(),
        mock_missing_manifest_set,
    );
    mock_discovered_missing.insert(
        "ordinary_parity_corpus".to_string(),
        DiscoveredSourceSet {
            name: "ordinary_parity_corpus".to_string(),
            paths: vec![],
            identity_count: 0,
            sha256_digest: "d1".to_string(),
            source_identities: vec![],
        },
    );
    mock_discovered_missing.insert(
        "tier1_ec_cases".to_string(),
        DiscoveredSourceSet {
            name: "tier1_ec_cases".to_string(),
            paths: vec![],
            identity_count: 0,
            sha256_digest: "d2".to_string(),
            source_identities: vec![],
        },
    );
    assert!(
        reconcile_source_sets_and_inventory(&mock_discovered_missing, &[], &[]).is_err(),
        "Canary 34 (missing-manifest disposition) FAILED"
    );

    // 35. Legacy param_types canary & ordinary row case suffix binding canary
    let mut legacy_param_types_row = dummy_row.clone();
    legacy_param_types_row.source_set = "ordinary_parity_corpus".to_string();
    legacy_param_types_row.source_identity =
        "projects/mamba/tests/harness/cpython/config/manifests/type-strict/param_types.toml#case=abs_rejects_str_argument"
            .to_string();
    legacy_param_types_row.fixture_or_probe_path =
        "projects/mamba/tests/cpython/type/core/param_types/abs_rejects_str_argument.py"
            .to_string();
    legacy_param_types_row.sut_command = format!(
        "{} run projects/mamba/tests/cpython/type/core/param_types/abs_rejects_str_argument.py",
        mamba_exe.to_string_lossy()
    );
    assert!(
        validate_oracle_hierarchy_row_fields(
            &legacy_param_types_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_ok(),
        "Canary 35a (legacy param_types binding) FAILED"
    );

    let mut mismatched_param_types_row = legacy_param_types_row.clone();
    mismatched_param_types_row.fixture_or_probe_path =
        "projects/mamba/tests/cpython/type/core/param_types/other_fixture.py".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &mismatched_param_types_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 35b (mismatched case suffix / fixture stem) FAILED"
    );

    // 36. Duplicate manifest-case canary
    let temp_dup_dir = tempfile::tempdir().expect("create temp dir for dup manifest canary");
    let manifest_rel_dir = temp_dup_dir
        .path()
        .join("projects/mamba/tests/harness/cpython/config/manifests");
    std::fs::create_dir_all(&manifest_rel_dir).expect("create temp manifest dir");
    std::fs::write(
        manifest_rel_dir.join("dup.toml"),
        r#"
bucket = "test"
lib = "test"
case = [
    { case = "dup_case", dimension = "type" },
    { case = "dup_case", dimension = "type" },
]
"#,
    )
    .expect("write dup manifest");
    assert!(
        discover_authoritative_source_sets(temp_dup_dir.path()).is_err(),
        "Canary 36a (duplicate manifest case identity) FAILED"
    );

    // 36b. Empty manifest case array canary
    let temp_empty_dir = tempfile::tempdir().expect("create temp dir for empty manifest canary");
    let manifest_empty_rel_dir = temp_empty_dir
        .path()
        .join("projects/mamba/tests/harness/cpython/config/manifests");
    std::fs::create_dir_all(&manifest_empty_rel_dir).expect("create temp manifest dir");
    std::fs::write(
        manifest_empty_rel_dir.join("empty.toml"),
        r#"
bucket = "test"
lib = "test"
case = []
"#,
    )
    .expect("write empty manifest");
    assert!(
        discover_authoritative_source_sets(temp_empty_dir.path()).is_err(),
        "Canary 36b (empty manifest case array) FAILED"
    );

    // 37. Malformed manifest identity canaries
    let mut malformed_manifest_row = legacy_param_types_row.clone();
    malformed_manifest_row.source_identity =
        "projects/mamba/tests/harness/cpython/config/manifests/type-strict/param_types.toml"
            .to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &malformed_manifest_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 37a (malformed manifest identity missing #case=) FAILED"
    );

    let mut empty_case_manifest_row = legacy_param_types_row.clone();
    empty_case_manifest_row.source_identity =
        "projects/mamba/tests/harness/cpython/config/manifests/type-strict/param_types.toml#case="
            .to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &empty_case_manifest_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 37b (malformed manifest identity empty case name) FAILED"
    );

    let mut non_toml_manifest_row = legacy_param_types_row.clone();
    non_toml_manifest_row.source_identity =
        "projects/mamba/tests/harness/cpython/config/manifests/type-strict/param_types.py#case=abs_rejects_str_argument"
            .to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &non_toml_manifest_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 37c (malformed manifest identity non-toml path) FAILED"
    );

    let mut parent_dir_manifest_row = legacy_param_types_row.clone();
    parent_dir_manifest_row.source_identity =
        "../param_types.toml#case=abs_rejects_str_argument".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &parent_dir_manifest_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 37d (malformed manifest identity parent dir) FAILED"
    );

    let mut abs_path_manifest_row = legacy_param_types_row.clone();
    abs_path_manifest_row.source_identity =
        "/abs/param_types.toml#case=abs_rejects_str_argument".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &abs_path_manifest_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 37e (malformed manifest identity absolute path) FAILED"
    );

    let mut dup_delim_manifest_row = legacy_param_types_row.clone();
    dup_delim_manifest_row.source_identity =
        "projects/mamba/tests/harness/cpython/config/manifests/type-strict/param_types.toml#case=abs_rejects_str_argument#case=extra"
            .to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &dup_delim_manifest_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 37f (malformed manifest identity duplicate #case= delimiter) FAILED"
    );

    let mut extra_fragment_manifest_row = legacy_param_types_row.clone();
    extra_fragment_manifest_row.source_identity =
        "projects/mamba/tests/harness/cpython/config/manifests/type-strict/param_types.toml#case=abs_rejects_str_argument#extra"
            .to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &extra_fragment_manifest_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 37g (malformed manifest identity extra fragment) FAILED"
    );

    let mut dot_dir_manifest_row = legacy_param_types_row.clone();
    dot_dir_manifest_row.source_identity =
        "./projects/mamba/tests/harness/cpython/config/manifests/type-strict/param_types.toml#case=abs_rejects_str_argument"
            .to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &dot_dir_manifest_row,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 37h (malformed manifest identity leading dot dir) FAILED"
    );

    // 38. Manifest path outside discovered ordinary-manifest set canary
    let mut mock_discovered_sets_c38 = BTreeMap::new();
    let canonical_id =
        "projects/mamba/tests/harness/cpython/config/manifests/type-strict/param_types.toml#case=abs_rejects_str_argument"
            .to_string();
    mock_discovered_sets_c38.insert(
        "ordinary_parity_corpus".to_string(),
        DiscoveredSourceSet {
            name: "ordinary_parity_corpus".to_string(),
            paths: vec![
                "projects/mamba/tests/harness/cpython/config/manifests/type-strict/param_types.toml"
                    .to_string(),
            ],
            identity_count: 1,
            sha256_digest: "digest_ord".to_string(),
            source_identities: vec![canonical_id.clone()],
        },
    );
    mock_discovered_sets_c38.insert(
        "tier1_ec_cases".to_string(),
        DiscoveredSourceSet {
            name: "tier1_ec_cases".to_string(),
            paths: vec![],
            identity_count: 0,
            sha256_digest: "digest_ec".to_string(),
            source_identities: vec![],
        },
    );
    mock_discovered_sets_c38.insert(
        "tier1_gate_denominators".to_string(),
        DiscoveredSourceSet {
            name: "tier1_gate_denominators".to_string(),
            paths: vec![],
            identity_count: 0,
            sha256_digest: "digest_denom".to_string(),
            source_identities: vec![],
        },
    );

    let canonical_row = legacy_param_types_row.clone();

    let mut outside_manifest_row = legacy_param_types_row.clone();
    outside_manifest_row.case_id = "outside_case".to_string();
    outside_manifest_row.probe_id = "outside_probe".to_string();
    outside_manifest_row.source_identity =
        "projects/mamba/tests/harness/cpython/config/manifests/type-strict/outside.toml#case=abs_rejects_str_argument"
            .to_string();
    outside_manifest_row.fixture_or_probe_path =
        "projects/mamba/tests/cpython/type/core/param_types/abs_rejects_str_argument.py"
            .to_string();

    let c38_res = reconcile_source_sets_and_inventory(
        &mock_discovered_sets_c38,
        &[],
        &[canonical_row, outside_manifest_row],
    );
    assert!(
        c38_res.is_err(),
        "Canary 38 (manifest outside discovered ordinary-manifest set) FAILED: expected error"
    );
    let c38_err = c38_res.unwrap_err();
    assert!(
        c38_err.contains("projects/mamba/tests/harness/cpython/config/manifests/type-strict/outside.toml#case=abs_rejects_str_argument"),
        "Canary 38 FAILED: error does not contain outside identity: {c38_err}"
    );
    assert!(
        c38_err.contains("undiscovered source_identity"),
        "Canary 38 FAILED: error does not contain 'undiscovered source_identity': {c38_err}"
    );

    // 39. Unknown tier1_dimension enum canary
    let mut bad_dim_enum = dummy_row.clone();
    bad_dim_enum.tier1_dimension = "unknown_dimension".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_dim_enum,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 39 (unknown tier1_dimension enum) FAILED"
    );

    // 40. Unknown channel enum canary
    let mut bad_chan_enum = dummy_row.clone();
    bad_chan_enum.channel = "unknown_channel".to_string();
    bad_chan_enum.expected_result_channel = "unknown_channel".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_chan_enum,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 40 (unknown channel enum) FAILED"
    );

    // 41. Unknown expected_terminal_classification enum canary
    let mut bad_term_enum = dummy_row.clone();
    bad_term_enum.expected_terminal_classification = "unknown_classification".to_string();
    bad_term_enum.sample_role = "unknown_classification".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_term_enum,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 41 (unknown expected_terminal_classification enum) FAILED"
    );

    // 42. Declaration mismatch canary
    let mut bad_decl_mismatch = dummy_row.clone();
    bad_decl_mismatch.sample_role = "green".to_string();
    bad_decl_mismatch.expected_terminal_classification = "intentional_red".to_string();
    assert!(
        validate_oracle_hierarchy_row_fields(
            &bad_decl_mismatch,
            &dummy_mamba_sha,
            dummy_git_head,
            dummy_platform,
            None
        )
        .is_err(),
        "Canary 42 (declaration mismatch) FAILED"
    );

    // 43. Observed mismatch canary
    let mut bad_obs_mismatch = dummy_row.clone();
    bad_obs_mismatch.expected_terminal_classification = "intentional_red".to_string();
    bad_obs_mismatch.sample_role = "intentional_red".to_string();
    bad_obs_mismatch.expected_outcome_kind = "ok".to_string();
    assert!(
        classify_and_reconcile_row(&bad_obs_mismatch, &dummy_ev_ok, &dummy_ev_ok).is_err(),
        "Canary 43 (observed mismatch) FAILED"
    );

    // 44. Missing dimension/classification coverage canary
    let make_rec = |case_id: &str, dim: &str, cls: &str, chan: &str| ObservedRowRecord {
        case_id: case_id.to_string(),
        channel: chan.to_string(),
        tier1_dimension: dim.to_string(),
        derived_classification: cls.to_string(),
        mamba_binary_sha256: dummy_mamba_sha.clone(),
        mamba_git_sha: dummy_git_head.to_string(),
        platform: dummy_platform.to_string(),
        oracle_executable: dummy_row.oracle_executable.clone(),
        oracle_version: dummy_row.oracle_version.clone(),
        inventory_sha256: "inv_sha".to_string(),
        oracle_ev: dummy_ev_ok.clone(),
        sut_ev: dummy_ev_ok.clone(),
    };

    let dataset_missing_cov = vec![
        make_rec("c1", "behavior", "green", "behavior"),
        make_rec("c2", "behavior", "intentional_red", "compile"),
        make_rec("c3", "stability", "green", "concurrency"),
        make_rec("c4", "stability", "intentional_red", "performance"),
        make_rec("c5", "efficiency", "green", "behavior"),
    ];
    assert!(
        reconcile_observed_dataset(&dataset_missing_cov, 5, "inv_sha").is_err(),
        "Canary 44 (missing dimension/classification coverage) FAILED"
    );

    // 45. Zero channel canary
    let dataset_zero_chan = vec![
        make_rec("c1", "behavior", "green", "behavior"),
        make_rec("c2", "behavior", "intentional_red", "compile"),
        make_rec("c3", "stability", "green", "behavior"),
        make_rec("c4", "stability", "intentional_red", "performance"),
        make_rec("c5", "efficiency", "green", "behavior"),
        make_rec("c6", "efficiency", "intentional_red", "compile"),
    ];
    assert!(
        reconcile_observed_dataset(&dataset_zero_chan, 6, "inv_sha").is_err(),
        "Canary 45 (zero channel) FAILED"
    );

    // 46. Identity differing stdout rejects canary
    let mut identity_diff_row = dummy_row.clone();
    identity_diff_row.oracle_kind = "cpython312_identity".to_string();
    identity_diff_row.expected_outcome_kind = "ok".to_string();
    let ev_identity_oracle = ExecutedCommandEvidence {
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        stdout_sha256: "sha_oracle".to_string(),
        stderr_sha256: "sha_err".to_string(),
        stdout_text: "ANCHOR 1".to_string(),
        stderr_text: "".to_string(),
    };
    let ev_identity_sut_diff = ExecutedCommandEvidence {
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        stdout_sha256: "sha_sut_different".to_string(),
        stderr_sha256: "sha_err".to_string(),
        stdout_text: "ANCHOR 2".to_string(),
        stderr_text: "".to_string(),
    };
    assert!(
        classify_and_reconcile_row(
            &identity_diff_row,
            &ev_identity_oracle,
            &ev_identity_sut_diff
        )
        .is_err(),
        "Canary 46 (identity differing stdout rejects) FAILED"
    );

    // 47. Property differing metric stdout with same anchor accepts green canary
    let mut prop_diff_row = dummy_row.clone();
    prop_diff_row.oracle_kind = "property".to_string();
    prop_diff_row.expected_outcome_kind = "ok".to_string();
    let ev_prop_oracle = ExecutedCommandEvidence {
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        stdout_sha256: "sha_oracle_metric1".to_string(),
        stderr_sha256: "sha_err".to_string(),
        stdout_text: "ANCHOR metric=100".to_string(),
        stderr_text: "".to_string(),
    };
    let ev_prop_sut_diff = ExecutedCommandEvidence {
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        stdout_sha256: "sha_sut_metric2".to_string(),
        stderr_sha256: "sha_err".to_string(),
        stdout_text: "ANCHOR metric=105".to_string(),
        stderr_text: "".to_string(),
    };
    assert_eq!(
        classify_and_reconcile_row(&prop_diff_row, &ev_prop_oracle, &ev_prop_sut_diff).unwrap(),
        "green",
        "Canary 47 (property differing metric stdout with same anchor accepts green) FAILED"
    );

    // 48. Property missing anchor or nonzero rejects canary
    let ev_prop_sut_no_anchor = ExecutedCommandEvidence {
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        stdout_sha256: "sha_sut_metric2".to_string(),
        stderr_sha256: "sha_err".to_string(),
        stdout_text: "NO_MARKER metric=105".to_string(),
        stderr_text: "".to_string(),
    };
    assert!(
        classify_and_reconcile_row(&prop_diff_row, &ev_prop_oracle, &ev_prop_sut_no_anchor)
            .is_err(),
        "Canary 48a (property missing anchor rejects) FAILED"
    );
    let ev_prop_sut_nonzero = ExecutedCommandEvidence {
        exit_code: Some(1),
        signal: None,
        timed_out: false,
        stdout_sha256: "sha_sut_metric2".to_string(),
        stderr_sha256: "sha_err".to_string(),
        stdout_text: "ANCHOR metric=105".to_string(),
        stderr_text: "".to_string(),
    };
    assert!(
        classify_and_reconcile_row(&prop_diff_row, &ev_prop_oracle, &ev_prop_sut_nonzero).is_err(),
        "Canary 48b (property nonzero rejects) FAILED"
    );

    // 49. Unknown route cannot relax canary
    let mut unknown_route_row = dummy_row.clone();
    unknown_route_row.oracle_kind = "unknown_route".to_string();
    unknown_route_row.expected_outcome_kind = "ok".to_string();
    let ev_same_stdout = ExecutedCommandEvidence {
        exit_code: Some(0),
        signal: None,
        timed_out: false,
        stdout_sha256: "sha_same".to_string(),
        stderr_sha256: "sha_err".to_string(),
        stdout_text: "ANCHOR".to_string(),
        stderr_text: "".to_string(),
    };
    assert!(
        classify_and_reconcile_row(&unknown_route_row, &ev_same_stdout, &ev_same_stdout).is_err(),
        "Canary 49 (unknown route cannot relax) FAILED"
    );

    // 50. Execution cwd canary proving a repo-root-relative command succeeds only under the supplied repo_root cwd
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));
    let rel_cmd = "test -f projects/mamba/tests/external_contracts/mamba_core_semantics_ec.rs";
    let root_ev = run_command_with_evidence(rel_cmd, repo_root, Duration::from_secs(5));
    assert_eq!(
        root_ev.exit_code,
        Some(0),
        "Canary 50 (repo-root-relative command under repo_root) FAILED"
    );
    let pkg_ev = run_command_with_evidence(rel_cmd, &manifest_dir, Duration::from_secs(5));
    assert_ne!(
        pkg_ev.exit_code,
        Some(0),
        "Canary 50 (repo-root-relative command under package dir must fail) FAILED"
    );
}

#[test]
fn oracle_hierarchy_and_result_identity() {
    run_oracle_hierarchy_in_test_mutation_canaries();

    let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/t1_oracle_hierarchy_inventory/manifest.toml");
    let cases_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/t1_oracle_hierarchy_inventory/cases.jsonl");
    let lock_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("external-contracts/evidence/mamba-t1-oracle-hierarchy-lock.json");
    let divergences_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/harness/cpython/config/type_divergences.txt");

    // 1. Verify existence of the three pinned artifacts
    assert!(
        manifest_path.is_file(),
        "Missing manifest.toml at {}",
        manifest_path.display()
    );
    assert!(
        cases_path.is_file(),
        "Missing cases.jsonl at {}",
        cases_path.display()
    );
    assert!(
        lock_path.is_file(),
        "Missing lock file at {}",
        lock_path.display()
    );

    // 2. Independently discover and digest authoritative source sets BEFORE reading cases.jsonl
    let mamba_manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = mamba_manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let discovered_sets =
        discover_authoritative_source_sets(repo_root).expect("discover authoritative source sets");
    assert_eq!(
        discovered_sets.len(),
        3,
        "must discover exactly 3 source sets"
    );
    for (set_name, set_data) in &discovered_sets {
        assert!(
            !set_data.paths.is_empty(),
            "source set '{set_name}' has 0 paths"
        );
        assert!(
            !set_data.sha256_digest.is_empty(),
            "source set '{set_name}' has empty digest"
        );
    }

    // 3. Resolve live Mamba binary and calculate actual non-empty SHA-256 hash
    let mamba_executable = mamba_bin();
    let mamba_path = std::path::Path::new(&mamba_executable);
    if !mamba_path.is_file() {
        panic!("Mamba binary missing at {}", mamba_path.display());
    }
    let mamba_bytes = std::fs::read(mamba_path).expect("read Mamba binary");
    if mamba_bytes.is_empty() {
        panic!("Mamba binary is 0 bytes at {}", mamba_path.display());
    }
    use sha2::{Digest, Sha256};
    let mut mamba_hasher = Sha256::new();
    mamba_hasher.update(&mamba_bytes);
    let live_mamba_sha256 = format!("{:x}", mamba_hasher.finalize());

    const EMPTY_FILE_SHA256: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    if live_mamba_sha256 == EMPTY_FILE_SHA256 {
        panic!("Mamba binary sha256 is the empty-file SHA-256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    // 4. Resolve live git HEAD revision
    let git_head_out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse HEAD");
    assert!(git_head_out.status.success(), "git rev-parse HEAD failed");
    let live_git_head = String::from_utf8_lossy(&git_head_out.stdout)
        .trim()
        .to_string();
    assert_eq!(
        live_git_head.len(),
        40,
        "live git HEAD must be 40 hex chars"
    );

    // 5. Resolve live target platform
    let arch = std::env::consts::ARCH;
    let os_vendor = match std::env::consts::OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        other => other,
    };
    let live_platform = format!("{arch}-{os_vendor}");

    // 6. Parse manifest.toml and reconcile identity
    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest.toml");
    let manifest: toml::Value = manifest_raw.parse().expect("parse manifest.toml");
    let row_count = manifest
        .get("row_count")
        .and_then(|v| v.as_integer())
        .expect("manifest row_count") as usize;
    let expected_inventory_sha256 = manifest
        .get("inventory_sha256")
        .and_then(|v| v.as_str())
        .expect("manifest inventory_sha256")
        .to_string();

    let manifest_mamba_sha = manifest
        .get("mamba_binary_sha256")
        .and_then(|v| v.as_str())
        .expect("manifest mamba_binary_sha256");
    if manifest_mamba_sha == EMPTY_FILE_SHA256 {
        panic!("manifest mamba_binary_sha256 is the empty-file SHA-256");
    }
    if live_mamba_sha256 != manifest_mamba_sha {
        panic!("Mamba binary sha256 mismatch: manifest expects {manifest_mamba_sha}, live binary is {live_mamba_sha256}");
    }

    let manifest_git_sha = manifest
        .get("mamba_git_sha")
        .and_then(|v| v.as_str())
        .expect("manifest mamba_git_sha");
    validate_source_revision_pin(manifest_git_sha, &live_git_head)
        .unwrap_or_else(|e| panic!("Source revision pin validation failed: {e}"));

    let manifest_platform = manifest
        .get("platform")
        .and_then(|v| v.as_str())
        .expect("manifest platform");
    if live_platform != manifest_platform {
        panic!("Platform mismatch: manifest expects {manifest_platform}, live platform is {live_platform}");
    }

    let manifest_pinned_sets = parse_pinned_source_set_records_from_toml(&manifest)
        .unwrap_or_else(|e| panic!("Manifest pinned source-set record parsing failed: {e}"));
    compare_pinned_and_discovered_source_sets(&manifest_pinned_sets, &discovered_sets)
        .unwrap_or_else(|e| panic!("Manifest pinned source-set validation failed: {e}"));

    // 7. Validate CPython executables declared in manifest.toml
    let c312_exe = manifest
        .get("cpython312_executable")
        .and_then(|v| v.as_str())
        .expect("cpython312_executable");
    let c312_ver = manifest
        .get("cpython312_version")
        .and_then(|v| v.as_str())
        .expect("cpython312_version");
    validate_cpython_executable_path_and_version(c312_exe, c312_ver);

    let c313t_exe = manifest
        .get("cpython313t_executable")
        .and_then(|v| v.as_str())
        .expect("cpython313t_executable");
    let c313t_ver = manifest
        .get("cpython313t_version")
        .and_then(|v| v.as_str())
        .expect("cpython313t_version");
    validate_cpython_executable_path_and_version(c313t_exe, c313t_ver);

    // 8. Compute and verify cases.jsonl sha256
    let cases_bytes = std::fs::read(&cases_path).expect("read cases.jsonl");
    let mut hasher = Sha256::new();
    hasher.update(&cases_bytes);
    let actual_cases_sha256 = format!("{:x}", hasher.finalize());
    assert_eq!(
        actual_cases_sha256, expected_inventory_sha256,
        "cases.jsonl sha256 mismatch"
    );

    // 9. Parse lock file and verify agreement
    let lock_raw = std::fs::read_to_string(&lock_path).expect("read lock file");
    let lock: serde_json::Value = serde_json::from_str(&lock_raw).expect("parse lock file");
    assert_eq!(
        lock.get("inventory_sha256").and_then(|v| v.as_str()),
        Some(expected_inventory_sha256.as_str())
    );
    assert_eq!(
        lock.get("row_count").and_then(|v| v.as_u64()),
        Some(row_count as u64)
    );

    let lock_pinned_sets = parse_pinned_source_set_records_from_json(&lock)
        .unwrap_or_else(|e| panic!("Lock pinned source-set record parsing failed: {e}"));
    compare_pinned_and_discovered_source_sets(&lock_pinned_sets, &discovered_sets)
        .unwrap_or_else(|e| panic!("Lock pinned source-set validation failed: {e}"));

    let dispositions_json = lock
        .get("out_of_scope_dispositions")
        .and_then(|v| v.as_array())
        .expect("out_of_scope_dispositions");
    let lock_dispositions: Vec<OutOfScopeDisposition> = dispositions_json
        .iter()
        .map(|d| OutOfScopeDisposition {
            source_set: d
                .get("source_set")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            source_identity: d
                .get("source_identity")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            reason: d
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            reviewed_against: d
                .get("reviewed_against")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect();

    let has_multicore_disposition = lock_dispositions.iter().any(|d| {
        d.source_identity
            .contains("t1_multicore_scaling_denominator")
            && d.reviewed_against == "#2022"
    });
    assert!(
        has_multicore_disposition,
        "Lock file missing out_of_scope disposition for t1_multicore_scaling_denominator reviewed against #2022"
    );

    // 10. Verify registry repair
    let div_raw = std::fs::read_to_string(&divergences_path).expect("read type_divergences.txt");
    assert!(
        !div_raw.contains("scandir_float_raises_typeerror.py"),
        "scandir_float_raises_typeerror.py must not survive as an intentional divergence"
    );

    // 11. Parse cases.jsonl rows, validate identity & execute subprocess engine
    let cases_str = String::from_utf8_lossy(&cases_bytes);
    let mut case_ids = std::collections::BTreeSet::new();
    let mut observed_records = Vec::new();
    let mut parsed_raw_rows = Vec::new();

    for line in cases_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let row_val: serde_json::Value = serde_json::from_str(trimmed)
            .unwrap_or_else(|e| panic!("invalid json line {trimmed:?}: {e}"));

        let row = RawCaseRow {
            case_id: row_val
                .get("case_id")
                .and_then(|v| v.as_str())
                .expect("case_id")
                .to_string(),
            channel: row_val
                .get("channel")
                .and_then(|v| v.as_str())
                .expect("channel")
                .to_string(),
            tier1_dimension: row_val
                .get("tier1_dimension")
                .and_then(|v| v.as_str())
                .expect("tier1_dimension")
                .to_string(),
            oracle_kind: row_val
                .get("oracle_kind")
                .and_then(|v| v.as_str())
                .expect("oracle_kind")
                .to_string(),
            oracle_executable: row_val
                .get("oracle_executable")
                .and_then(|v| v.as_str())
                .expect("oracle_executable")
                .to_string(),
            oracle_version: row_val
                .get("oracle_version")
                .and_then(|v| v.as_str())
                .expect("oracle_version")
                .to_string(),
            oracle_command: row_val
                .get("oracle_command")
                .and_then(|v| v.as_str())
                .expect("oracle_command")
                .to_string(),
            sut_command: row_val
                .get("sut_command")
                .and_then(|v| v.as_str())
                .expect("sut_command")
                .to_string(),
            expected_divergence_class: row_val
                .get("expected_divergence_class")
                .and_then(|v| v.as_str())
                .expect("expected_divergence_class")
                .to_string(),
            expected_outcome_kind: row_val
                .get("expected_outcome_kind")
                .and_then(|v| v.as_str())
                .unwrap_or("ok")
                .to_string(),
            expected_probe_anchor: row_val
                .get("expected_probe_anchor")
                .and_then(|v| v.as_str())
                .expect("expected_probe_anchor")
                .to_string(),
            expected_result_channel: row_val
                .get("expected_result_channel")
                .and_then(|v| v.as_str())
                .expect("expected_result_channel")
                .to_string(),
            expected_terminal_classification: row_val
                .get("expected_terminal_classification")
                .and_then(|v| v.as_str())
                .expect("expected_terminal_classification")
                .to_string(),
            diagnostic_class: row_val
                .get("diagnostic_class")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            diagnostic_span: row_val
                .get("diagnostic_span")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            fixture_or_probe_path: row_val
                .get("fixture_or_probe_path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            mamba_binary_sha256: row_val
                .get("mamba_binary_sha256")
                .and_then(|v| v.as_str())
                .expect("mamba_binary_sha256")
                .to_string(),
            mamba_git_sha: row_val
                .get("mamba_git_sha")
                .and_then(|v| v.as_str())
                .expect("mamba_git_sha")
                .to_string(),
            platform: row_val
                .get("platform")
                .and_then(|v| v.as_str())
                .expect("platform")
                .to_string(),
            source_set: row_val
                .get("source_set")
                .and_then(|v| v.as_str())
                .expect("source_set")
                .to_string(),
            source_identity: row_val
                .get("source_identity")
                .and_then(|v| v.as_str())
                .expect("source_identity")
                .to_string(),
            probe_id: row_val
                .get("probe_id")
                .and_then(|v| v.as_str())
                .expect("probe_id")
                .to_string(),
            sample_role: row_val
                .get("sample_role")
                .and_then(|v| v.as_str())
                .expect("sample_role")
                .to_string(),
        };

        assert!(
            case_ids.insert(row.case_id.clone()),
            "duplicate case_id: {}",
            row.case_id
        );

        parsed_raw_rows.push(row.clone());

        validate_oracle_hierarchy_row_fields(
            &row,
            &live_mamba_sha256,
            manifest_git_sha,
            &live_platform,
            Some(&discovered_sets),
        )
        .unwrap_or_else(|e| panic!("{e}"));

        validate_cpython_executable_path_and_version(
            &row.oracle_executable,
            &row.oracle_version.replace("Python ", ""),
        );

        let oracle_ev =
            run_command_with_evidence(&row.oracle_command, repo_root, Duration::from_secs(30));
        let sut_ev =
            run_command_with_evidence(&row.sut_command, repo_root, Duration::from_secs(30));

        let derived_cls =
            classify_and_reconcile_row(&row, &oracle_ev, &sut_ev).unwrap_or_else(|e| panic!("{e}"));

        observed_records.push(ObservedRowRecord {
            case_id: row.case_id,
            channel: row.channel,
            tier1_dimension: row.tier1_dimension,
            derived_classification: derived_cls,
            mamba_binary_sha256: row.mamba_binary_sha256,
            mamba_git_sha: row.mamba_git_sha,
            platform: row.platform,
            oracle_executable: row.oracle_executable,
            oracle_version: row.oracle_version,
            inventory_sha256: expected_inventory_sha256.clone(),
            oracle_ev,
            sut_ev,
        });
    }

    reconcile_source_sets_and_inventory(&discovered_sets, &lock_dispositions, &parsed_raw_rows)
        .unwrap_or_else(|e| panic!("{e}"));

    let (green_count, red_count, channel_counts) =
        reconcile_observed_dataset(&observed_records, row_count, &expected_inventory_sha256)
            .unwrap_or_else(|e| panic!("{e}"));

    println!(
        "MAMBA-T1-ORACLE-HIERARCHY-AND-RESULT-IDENTITY PASS selected={} executed={} green={} intentional_red={} channel_compile={} channel_behavior={} channel_concurrency={} channel_performance={}",
        row_count,
        observed_records.len(),
        green_count,
        red_count,
        channel_counts.get("compile").copied().unwrap_or(0),
        channel_counts.get("behavior").copied().unwrap_or(0),
        channel_counts.get("concurrency").copied().unwrap_or(0),
        channel_counts.get("performance").copied().unwrap_or(0),
    );
}
