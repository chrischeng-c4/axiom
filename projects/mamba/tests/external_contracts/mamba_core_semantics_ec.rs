// SPEC-MANAGED: executable evidence for the Mamba Tier 1 to_thread/gather ECs.
// @ec mamba-t1-to-thread-gather-results
// @ec mamba-t1-to-thread-gather-stability
// @ec mamba-t1-to-thread-gather-efficiency

#![cfg(unix)]

use std::collections::BTreeSet;
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
        (0_u32..100).collect(),
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
