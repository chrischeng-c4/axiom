// SPEC-MANAGED: projects/cap/tech-design/semantic/source/projects-cap-benches-command_resources-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Resource benchmarks for same-name `cap <command>` replacements.
//!
//! Measures the actual CLI process users run: `cap <cmd>` versus the original
//! command. CPU is `user + system` time from child `rusage`; memory is peak RSS.

use std::{
    cmp::Ordering,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use serde::Serialize;

const DEFAULT_ROUNDS: usize = 7;
const DEFAULT_WARMUPS: usize = 2;

#[derive(Debug)]
struct Scenario {
    id: &'static str,
    command: &'static str,
    description: &'static str,
    gate: Gate,
    expected_exit_code: i32,
    cap_args: Vec<String>,
    original_program: String,
    original_args: Vec<String>,
    stdin_file: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
enum Gate {
    DualWin,
    RssFallback,
    Candidate,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#e2e-test
impl Gate {
    fn label(self) -> &'static str {
        match self {
            Self::DualWin => "dual-win",
            Self::RssFallback => "rss-fallback",
            Self::Candidate => "candidate",
        }
    }

    fn is_gated(self) -> bool {
        !matches!(self, Self::Candidate)
    }

    fn failure_reason(self, cpu_ratio: f64, rss_ratio: f64) -> Option<&'static str> {
        match self {
            Self::DualWin if cpu_ratio >= 1.0 || rss_ratio >= 1.0 => {
                Some("dual-win requires CPU and RSS below original")
            }
            Self::RssFallback if rss_ratio >= 1.0 => {
                Some("rss-fallback requires RSS below original")
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct Measurement {
    exit_code: i32,
    user_cpu_us: u64,
    system_cpu_us: u64,
    total_cpu_us: u64,
    peak_rss_bytes: u64,
}

#[derive(Debug, Serialize)]
struct ScenarioReport {
    id: String,
    command: String,
    description: String,
    gate: Gate,
    rounds: usize,
    warmups: usize,
    cap: Measurement,
    original: Measurement,
    cpu_ratio_cap_over_original: f64,
    peak_rss_ratio_cap_over_original: f64,
}

#[derive(Debug, Serialize)]
struct BenchReport {
    metric: String,
    rounds: usize,
    warmups: usize,
    scenarios: Vec<ScenarioReport>,
}

fn main() -> Result<()> {
    let rounds = env_usize("CAP_BENCH_ROUNDS", DEFAULT_ROUNDS);
    let warmups = env_usize("CAP_BENCH_WARMUPS", DEFAULT_WARMUPS);
    let include_candidates = env_bool("CAP_BENCH_INCLUDE_CANDIDATES");
    let command_filter = env_command_filter("CAP_BENCH_COMMANDS");
    let cap = cap_binary()?;
    let fixture = Fixture::create()?;
    let scenarios = fixture
        .scenarios()
        .into_iter()
        .filter(|scenario| {
            command_filter
                .as_ref()
                .map(|commands| commands.iter().any(|command| command == scenario.command))
                .unwrap_or(scenario.gate.is_gated() || include_candidates)
        })
        .collect::<Vec<_>>();

    println!(
        "cap command resource benchmark: rounds={rounds} warmups={warmups} cap={}",
        cap.display()
    );
    println!(
        "| command | gate | scenario | cap cpu ms | original cpu ms | cpu ratio | cap rss MiB | original rss MiB | rss ratio |"
    );
    println!("|---|---:|---|---:|---:|---:|---:|---:|---:|");

    let mut reports = Vec::new();
    let mut failing_gated = Vec::new();
    for scenario in scenarios {
        let cap_measurement = measure_median(
            &cap,
            &scenario.cap_args,
            scenario.stdin_file.as_deref(),
            warmups,
            rounds,
        )
        .with_context(|| format!("measuring cap {}", scenario.id))?;
        let original_measurement = measure_median(
            Path::new(&scenario.original_program),
            &scenario.original_args,
            scenario.stdin_file.as_deref(),
            warmups,
            rounds,
        )
        .with_context(|| format!("measuring original {}", scenario.id))?;

        if cap_measurement.exit_code != scenario.expected_exit_code
            || original_measurement.exit_code != scenario.expected_exit_code
        {
            bail!(
                "scenario {} failed: cap exit {}, original exit {}, expected {}",
                scenario.id,
                cap_measurement.exit_code,
                original_measurement.exit_code,
                scenario.expected_exit_code
            );
        }

        let cpu_ratio = ratio(
            cap_measurement.total_cpu_us,
            original_measurement.total_cpu_us,
        );
        let rss_ratio = ratio(
            cap_measurement.peak_rss_bytes,
            original_measurement.peak_rss_bytes,
        );

        println!(
            "| `{}` | {} | {} | {:.3} | {:.3} | {:.2}x | {:.2} | {:.2} | {:.2}x |",
            scenario.command,
            scenario.gate.label(),
            scenario.description,
            us_to_ms(cap_measurement.total_cpu_us),
            us_to_ms(original_measurement.total_cpu_us),
            cpu_ratio,
            bytes_to_mib(cap_measurement.peak_rss_bytes),
            bytes_to_mib(original_measurement.peak_rss_bytes),
            rss_ratio,
        );

        if let Some(reason) = scenario.gate.failure_reason(cpu_ratio, rss_ratio) {
            failing_gated.push(format!(
                "{} gate={} cpu={cpu_ratio:.2}x rss={rss_ratio:.2}x ({reason})",
                scenario.id,
                scenario.gate.label()
            ));
        }

        reports.push(ScenarioReport {
            id: scenario.id.to_string(),
            command: scenario.command.to_string(),
            description: scenario.description.to_string(),
            gate: scenario.gate,
            rounds,
            warmups,
            cap: cap_measurement,
            original: original_measurement,
            cpu_ratio_cap_over_original: cpu_ratio,
            peak_rss_ratio_cap_over_original: rss_ratio,
        });
    }

    let report = BenchReport {
        metric: "median child rusage: total_cpu_us=user+system, peak_rss_bytes=platform-normalized maxrss".to_string(),
        rounds,
        warmups,
        scenarios: reports,
    };
    write_reports(&report)?;
    if !failing_gated.is_empty() {
        bail!(
            "gated cap replacements must satisfy their resource policy; failing: {}",
            failing_gated.join(", ")
        );
    }
    Ok(())
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn env_bool(name: &str) -> bool {
    env::var(name)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn env_command_filter(name: &str) -> Option<Vec<String>> {
    let value = env::var(name).ok()?;
    let commands = value
        .split(',')
        .map(str::trim)
        .filter(|command| !command.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    (!commands.is_empty()).then_some(commands)
}

fn cap_binary() -> Result<PathBuf> {
    if let Ok(path) = env::var("CAP_BENCH_CAP_BINARY") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        bail!(
            "CAP_BENCH_CAP_BINARY does not point to a file: {}",
            path.display()
        );
    }

    // The installed/released `cap` is always the C front-end (cap_frontend.c +
    // cap_fast_frontend.c); only `cap-full` is Rust. The bench measures that
    // production shape, so it builds the C front-end directly.
    build_c_frontend()
}

fn build_c_frontend() -> Result<PathBuf> {
    let out_dir = PathBuf::from("target");
    fs::create_dir_all(&out_dir).context("creating target output directory")?;
    let out = out_dir.join(format!(
        "cap-command-resource-frontend{}",
        env::consts::EXE_SUFFIX
    ));
    let fast = out_dir.join(format!("cap-fast{}", env::consts::EXE_SUFFIX));
    let full = out_dir.join(format!("cap-full{}", env::consts::EXE_SUFFIX));
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/cap_frontend.c");
    let fast_source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/cap_fast_frontend.c");
    let strip_flag = if cfg!(target_os = "macos") {
        "-Wl,-dead_strip"
    } else {
        "-Wl,--gc-sections"
    };
    let c_flags = [
        "-Oz",
        "-ffunction-sections",
        "-fdata-sections",
        "-fno-stack-protector",
        "-fno-unwind-tables",
        "-fno-asynchronous-unwind-tables",
        strip_flag,
    ];
    let mut frontend_flags = c_flags.to_vec();
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        frontend_flags.extend([
            "-ffreestanding",
            "-fno-builtin",
            "-nostartfiles",
            "-Wl,-e,_start",
        ]);
    }
    let fast_status = Command::new("/usr/bin/cc")
        .args(c_flags)
        .arg(&fast_source)
        .arg("-o")
        .arg(&fast)
        .status()
        .with_context(|| format!("building {}", fast_source.display()))?;
    if !fast_status.success() {
        bail!(
            "building C cap-fast front-end failed with status {fast_status}: {}",
            fast_source.display()
        );
    }
    ensure_cap_full_sibling(&full)?;
    let status = Command::new("/usr/bin/cc")
        .args(&frontend_flags)
        .arg(&source)
        .arg("-o")
        .arg(&out)
        .status()
        .with_context(|| format!("building {}", source.display()))?;
    if !status.success() {
        bail!(
            "building C cap front-end failed with status {status}: {}",
            source.display()
        );
    }
    if cfg!(target_os = "macos") {
        let _ = Command::new("codesign")
            .args(["-s", "-", "-f", "--options", "runtime"])
            .arg(&out)
            .status();
        let _ = Command::new("codesign")
            .args(["-s", "-", "-f", "--options", "runtime"])
            .arg(&fast)
            .status();
    }
    Ok(out)
}

fn ensure_cap_full_sibling(full: &Path) -> Result<()> {
    let source = locate_cap_full_binary()?;
    fs::copy(&source, full)
        .with_context(|| format!("copying {} to {}", source.display(), full.display()))?;
    if cfg!(target_os = "macos") {
        let _ = Command::new("codesign")
            .args(["-s", "-", "-f"])
            .arg(full)
            .status();
    }
    Ok(())
}

fn locate_cap_full_binary() -> Result<PathBuf> {
    if let Some(path) = option_env!("CARGO_BIN_EXE_cap-full") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    let current = env::current_exe().context("resolve current bench executable")?;
    let Some(deps_dir) = current.parent() else {
        bail!("bench executable has no parent path: {}", current.display());
    };
    let Some(profile_dir) = deps_dir.parent() else {
        bail!(
            "bench executable has no profile path: {}",
            current.display()
        );
    };
    let candidate = profile_dir.join(format!("cap-full{}", env::consts::EXE_SUFFIX));
    if candidate.is_file() {
        return Ok(candidate);
    }

    bail!(
        "could not locate cap-full binary; run `cargo build -p cap --bin cap-full --release` before candidate benchmarks"
    )
}

fn measure_median(
    program: &Path,
    args: &[String],
    stdin_file: Option<&Path>,
    warmups: usize,
    rounds: usize,
) -> Result<Measurement> {
    for _ in 0..warmups {
        let measurement = measure_once(program, args, stdin_file)?;
        if measurement.exit_code != 0 {
            return Ok(measurement);
        }
    }

    let mut measurements = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        measurements.push(measure_once(program, args, stdin_file)?);
    }
    measurements.sort_by(compare_measurement);
    Ok(measurements[measurements.len() / 2].clone())
}

fn compare_measurement(left: &Measurement, right: &Measurement) -> Ordering {
    left.total_cpu_us
        .cmp(&right.total_cpu_us)
        .then_with(|| left.peak_rss_bytes.cmp(&right.peak_rss_bytes))
}

fn measure_once(program: &Path, args: &[String], stdin_file: Option<&Path>) -> Result<Measurement> {
    let stdin = match stdin_file {
        Some(path) => Stdio::from(
            fs::File::open(path).with_context(|| format!("opening stdin {}", path.display()))?,
        ),
        None => Stdio::null(),
    };
    let child = Command::new(program)
        .args(args)
        .stdin(stdin)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawning {}", render_command(program, args)))?;
    let pid = child.id() as libc::pid_t;
    std::mem::forget(child);

    let mut status = 0;
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    let waited = unsafe { libc::wait4(pid, &mut status, 0, usage.as_mut_ptr()) };
    if waited < 0 {
        return Err(io::Error::last_os_error())
            .with_context(|| format!("wait4 {}", render_command(program, args)));
    }
    let usage = unsafe { usage.assume_init() };
    let user_cpu_us = timeval_us(usage.ru_utime);
    let system_cpu_us = timeval_us(usage.ru_stime);

    Ok(Measurement {
        exit_code: exit_code(status),
        user_cpu_us,
        system_cpu_us,
        total_cpu_us: user_cpu_us + system_cpu_us,
        peak_rss_bytes: maxrss_bytes(usage.ru_maxrss),
    })
}

fn timeval_us(value: libc::timeval) -> u64 {
    (value.tv_sec as u64 * 1_000_000) + value.tv_usec as u64
}

fn exit_code(status: i32) -> i32 {
    if libc::WIFEXITED(status) {
        libc::WEXITSTATUS(status)
    } else if libc::WIFSIGNALED(status) {
        128 + libc::WTERMSIG(status)
    } else {
        status
    }
}

#[cfg(target_os = "linux")]
fn maxrss_bytes(raw: libc::c_long) -> u64 {
    raw.max(0) as u64 * 1024
}

#[cfg(not(target_os = "linux"))]
fn maxrss_bytes(raw: libc::c_long) -> u64 {
    raw.max(0) as u64
}

fn write_reports(report: &BenchReport) -> Result<()> {
    let out_dir = PathBuf::from("target");
    fs::create_dir_all(&out_dir).context("creating target output directory")?;
    let json_path = out_dir.join("cap-command-resource-bench.json");
    let md_path = out_dir.join("cap-command-resource-bench.md");
    fs::write(&json_path, serde_json::to_string_pretty(report)?)
        .with_context(|| format!("writing {}", json_path.display()))?;
    fs::write(&md_path, report_markdown(report))
        .with_context(|| format!("writing {}", md_path.display()))?;
    println!("\nwrote {}", json_path.display());
    println!("wrote {}", md_path.display());
    Ok(())
}

fn report_markdown(report: &BenchReport) -> String {
    let mut out = String::new();
    out.push_str("# Cap Command Resource Benchmark\n\n");
    out.push_str(&format!(
        "Metric: {}. Rounds: {}. Warmups: {}.\n\n",
        report.metric, report.rounds, report.warmups
    ));
    out.push_str("| Command | Gate | Scenario | Cap CPU ms | Original CPU ms | CPU Ratio | Cap RSS MiB | Original RSS MiB | RSS Ratio |\n");
    out.push_str("|---|---:|---|---:|---:|---:|---:|---:|---:|\n");
    for scenario in &report.scenarios {
        out.push_str(&format!(
            "| `{}` | {} | {} | {:.3} | {:.3} | {:.2}x | {:.2} | {:.2} | {:.2}x |\n",
            scenario.command,
            scenario.gate.label(),
            scenario.description,
            us_to_ms(scenario.cap.total_cpu_us),
            us_to_ms(scenario.original.total_cpu_us),
            scenario.cpu_ratio_cap_over_original,
            bytes_to_mib(scenario.cap.peak_rss_bytes),
            bytes_to_mib(scenario.original.peak_rss_bytes),
            scenario.peak_rss_ratio_cap_over_original,
        ));
    }
    out
}

fn ratio(left: u64, right: u64) -> f64 {
    if right == 0 {
        return f64::INFINITY;
    }
    left as f64 / right as f64
}

fn us_to_ms(us: u64) -> f64 {
    us as f64 / 1_000.0
}

fn bytes_to_mib(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn render_command(program: &Path, args: &[String]) -> String {
    std::iter::once(program.display().to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#e2e-test
impl Fixture {
    fn create() -> Result<Self> {
        let dir = tempfile::tempdir().context("creating cap benchmark fixture")?;
        let root = dir.path().to_path_buf();

        let list_dir = root.join("ls-many");
        fs::create_dir(&list_dir)?;
        for idx in 0..20_000 {
            fs::write(list_dir.join(format!("file-{idx:04}.txt")), b"x")?;
        }
        for idx in 0..500 {
            fs::write(list_dir.join(format!(".hidden-{idx:03}")), b"x")?;
        }

        let cat_file = root.join("cat-large.txt");
        write_repeated(&cat_file, b"0123456789abcdef\n", 512 * 1024)?;

        let mkdir_existing = root.join("mkdir-existing/a/b");
        fs::create_dir_all(&mkdir_existing)?;

        let touch_file = root.join("touch-existing.txt");
        fs::write(&touch_file, b"touch\n")?;

        let byte_window_file = root.join("byte-window.bin");
        let byte_window_chunk = vec![b'x'; 64 * 1024];
        write_repeated(&byte_window_file, &byte_window_chunk, 1024)?;

        let sed_file = root.join("sed-lines.txt");
        let mut sed = fs::File::create(&sed_file)?;
        for idx in 1..=120_000 {
            if idx % 100 == 0 {
                writeln!(sed, "line {idx:06} NEEDLE")?;
            } else {
                writeln!(sed, "line {idx:06}")?;
            }
        }

        let sort_file = root.join("sort-lines.txt");
        let mut sort = fs::File::create(&sort_file)?;
        for idx in (0..500_000).rev() {
            writeln!(sort, "line-{idx:06}")?;
        }

        let find_root = root.join("find-tree");
        for dir_idx in 0..80 {
            let subdir = find_root.join(format!("dir-{dir_idx:03}"));
            fs::create_dir_all(&subdir)?;
            for file_idx in 0..20 {
                fs::write(subdir.join(format!("item-{file_idx:03}.txt")), b"find\n")?;
                fs::write(subdir.join(format!("item-{file_idx:03}.bin")), b"find\n")?;
            }
        }

        let grep_root = root.join("grep-tree");
        for dir_idx in 0..40 {
            let subdir = grep_root.join(format!("dir-{dir_idx:03}"));
            fs::create_dir_all(&subdir)?;
            for file_idx in 0..20 {
                let file = subdir.join(format!("search-{file_idx:03}.txt"));
                let mut contents = String::new();
                for line_idx in 0..80 {
                    if (dir_idx + file_idx + line_idx) % 97 == 0 {
                        contents.push_str("NEEDLE found here\n");
                    } else {
                        contents.push_str("ordinary searchable text\n");
                    }
                }
                fs::write(file, contents)?;
            }
        }

        let xargs_input = root.join("xargs-input.txt");
        let mut xargs = fs::File::create(&xargs_input)?;
        for idx in 0..20_000 {
            writeln!(xargs, "item-{idx:05}")?;
        }

        Ok(Self { _dir: dir, root })
    }

    fn scenarios(&self) -> Vec<Scenario> {
        let list_dir = self.root.join("ls-many");
        let cat_file = self.root.join("cat-large.txt");
        let mkdir_existing = self.root.join("mkdir-existing/a/b");
        let touch_file = self.root.join("touch-existing.txt");
        let byte_window_file = self.root.join("byte-window.bin");
        let sed_file = self.root.join("sed-lines.txt");
        let sort_file = self.root.join("sort-lines.txt");
        let find_root = self.root.join("find-tree");
        let grep_root = self.root.join("grep-tree");
        let xargs_input = self.root.join("xargs-input.txt");
        let long_basename_suffix = "suffix".repeat(78);
        let long_basename_path = format!(
            "/tmp/cap/bench/{}file{}",
            "nested/".repeat(78),
            long_basename_suffix
        );
        let long_dirname_path = format!("/tmp/cap/bench/{}file.txt", "nested/".repeat(140));
        let grep_head_pipe = format!("grep -R NEEDLE {} | head -n 50", path_string(&grep_root));
        let awk_xargs_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | xargs echo",
            path_string(&sed_file)
        );
        let find_xargs_pipe = format!(
            "find {} -type f -name '*.txt' | xargs wc -l",
            path_string(&find_root)
        );
        let run_ls = format!("ls -1 {}", path_string(&list_dir));
        let run_cat = format!("cat {}", path_string(&cat_file));
        let run_uniq = format!("uniq {}", path_string(&byte_window_file));
        let run_find = format!("find {} -type f -name '*.txt'", path_string(&find_root));
        let run_du = format!("du -sk {}", path_string(&find_root));
        let run_sort = format!("sort {}", path_string(&sort_file));
        let run_sed = format!("sed -n 2500,7500p {}", path_string(&sed_file));
        let run_grep = format!("grep -R NEEDLE {}", path_string(&grep_root));

        vec![
            Scenario {
                id: "true_noop",
                command: "true",
                description: "zero-argument success exit",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["true"]),
                original_program: "/usr/bin/true".to_string(),
                original_args: vec![],
                stdin_file: None,
            },
            Scenario {
                id: "false_noop",
                command: "false",
                description: "zero-argument failure exit",
                gate: Gate::Candidate,
                expected_exit_code: 1,
                cap_args: strings(["false"]),
                original_program: "/usr/bin/false".to_string(),
                original_args: vec![],
                stdin_file: None,
            },
            Scenario {
                id: "pwd_current",
                command: "pwd",
                description: "print current directory",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["pwd"]),
                original_program: "/bin/pwd".to_string(),
                original_args: vec![],
                stdin_file: None,
            },
            Scenario {
                id: "basename_path",
                command: "basename",
                description: "long path basename with suffix",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["basename", &long_basename_path, &long_basename_suffix]),
                original_program: "/usr/bin/basename".to_string(),
                original_args: strings([&long_basename_path, &long_basename_suffix]),
                stdin_file: None,
            },
            Scenario {
                id: "dirname_path",
                command: "dirname",
                description: "long path dirname",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["dirname", &long_dirname_path]),
                original_program: "/usr/bin/dirname".to_string(),
                original_args: strings([&long_dirname_path]),
                stdin_file: None,
            },
            Scenario {
                id: "ls_many",
                command: "ls",
                description: "20,000 visible entries",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["ls", "-1", &path_string(&list_dir)]),
                original_program: "/bin/ls".to_string(),
                original_args: strings(["-1", &path_string(&list_dir)]),
                stdin_file: None,
            },
            Scenario {
                id: "run_string_ls_many",
                command: "run",
                description: "hook string: ls 20,000 visible entries",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_ls]),
                original_program: "/bin/ls".to_string(),
                original_args: strings(["-1", &path_string(&list_dir)]),
                stdin_file: None,
            },
            Scenario {
                id: "mkdir_existing_p",
                command: "mkdir",
                description: "idempotent mkdir -p existing deep directory",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["mkdir", "-p", &path_string(&mkdir_existing)]),
                original_program: "/bin/mkdir".to_string(),
                original_args: strings(["-p", &path_string(&mkdir_existing)]),
                stdin_file: None,
            },
            Scenario {
                id: "touch_existing",
                command: "touch",
                description: "touch existing regular file",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["touch", &path_string(&touch_file)]),
                original_program: "/usr/bin/touch".to_string(),
                original_args: strings([&path_string(&touch_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "cat_large",
                command: "cat",
                description: "8.5 MiB regular file",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["cat", &path_string(&cat_file)]),
                original_program: "/bin/cat".to_string(),
                original_args: strings([&path_string(&cat_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "run_string_cat_large",
                command: "run",
                description: "hook string: cat 8.5 MiB regular file",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_cat]),
                original_program: "/bin/cat".to_string(),
                original_args: strings([&path_string(&cat_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "head_byte_window",
                command: "head",
                description: "first 64 MiB byte window",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["head", "-c", "67108864", &path_string(&byte_window_file)]),
                original_program: "/usr/bin/head".to_string(),
                original_args: strings(["-c", "67108864", &path_string(&byte_window_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "tail_byte_window",
                command: "tail",
                description: "last 64 MiB byte window",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["tail", "-c", "67108864", &path_string(&byte_window_file)]),
                original_program: "/usr/bin/tail".to_string(),
                original_args: strings(["-c", "67108864", &path_string(&byte_window_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "uniq_long_line",
                command: "uniq",
                description: "64 MiB single-line file",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["uniq", &path_string(&byte_window_file)]),
                original_program: "/usr/bin/uniq".to_string(),
                original_args: strings([&path_string(&byte_window_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "run_string_uniq_long_line",
                command: "run",
                description: "hook string: uniq 64 MiB single-line file",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_uniq]),
                original_program: "/usr/bin/uniq".to_string(),
                original_args: strings([&path_string(&byte_window_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "find_name_type",
                command: "find",
                description: "3,200 files, -type f -name *.txt",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings([
                    "find",
                    &path_string(&find_root),
                    "-type",
                    "f",
                    "-name",
                    "*.txt",
                ]),
                original_program: "/usr/bin/find".to_string(),
                original_args: strings([&path_string(&find_root), "-type", "f", "-name", "*.txt"]),
                stdin_file: None,
            },
            Scenario {
                id: "run_string_find_name_type",
                command: "run",
                description: "hook string: find 3,200 files, -type f -name *.txt",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_find]),
                original_program: "/usr/bin/find".to_string(),
                original_args: strings([&path_string(&find_root), "-type", "f", "-name", "*.txt"]),
                stdin_file: None,
            },
            Scenario {
                id: "du_summary_kib",
                command: "du",
                description: "summary KiB for 3,200-file tree",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["du", "-sk", &path_string(&find_root)]),
                original_program: "/usr/bin/du".to_string(),
                original_args: strings(["-sk", &path_string(&find_root)]),
                stdin_file: None,
            },
            Scenario {
                id: "run_string_du_summary_kib",
                command: "run",
                description: "hook string: du summary KiB for 3,200-file tree",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_du]),
                original_program: "/usr/bin/du".to_string(),
                original_args: strings(["-sk", &path_string(&find_root)]),
                stdin_file: None,
            },
            Scenario {
                id: "sort_single_file",
                command: "sort",
                description: "500,000 reverse-sorted lines",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["sort", &path_string(&sort_file)]),
                original_program: "/usr/bin/sort".to_string(),
                original_args: strings([&path_string(&sort_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "run_string_sort_single_file",
                command: "run",
                description: "hook string: sort 500,000 reverse-sorted lines",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_sort]),
                original_program: "/usr/bin/sort".to_string(),
                original_args: strings([&path_string(&sort_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "sed_range",
                command: "sed",
                description: "print 5,001 lines from 120,000-line file",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["sed", "-n", "2500,7500p", &path_string(&sed_file)]),
                original_program: "/usr/bin/sed".to_string(),
                original_args: strings(["-n", "2500,7500p", &path_string(&sed_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "run_string_sed_range",
                command: "run",
                description: "hook string: sed print 5,001 lines from 120,000-line file",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_sed]),
                original_program: "/usr/bin/sed".to_string(),
                original_args: strings(["-n", "2500,7500p", &path_string(&sed_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "awk_count_matches",
                command: "awk",
                description: "count NEEDLE matches in 120,000-line file",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings([
                    "awk",
                    "/NEEDLE/ { c++ } END { print c }",
                    &path_string(&sed_file),
                ]),
                original_program: "/usr/bin/awk".to_string(),
                original_args: strings([
                    "/NEEDLE/ { c++ } END { print c }",
                    &path_string(&sed_file),
                ]),
                stdin_file: None,
            },
            Scenario {
                id: "xargs_echo_words",
                command: "xargs",
                description: "xargs echo over 20,000 input words",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["xargs", "echo"]),
                original_program: "/usr/bin/xargs".to_string(),
                original_args: strings(["echo"]),
                stdin_file: Some(xargs_input.clone()),
            },
            Scenario {
                id: "pipe_grep_head",
                command: "pipe",
                description: "grep -R piped to head",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_xargs",
                command: "pipe",
                description: "awk output piped to xargs echo",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_xargs",
                command: "pipe",
                description: "find results piped to xargs wc -l",
                gate: Gate::Candidate,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "grep_recursive",
                command: "grep",
                description: "800 text files, recursive literal search",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["grep", "-R", "NEEDLE", &path_string(&grep_root)]),
                original_program: "/usr/bin/grep".to_string(),
                original_args: strings(["-R", "NEEDLE", &path_string(&grep_root)]),
                stdin_file: None,
            },
            Scenario {
                id: "run_string_grep_recursive",
                command: "run",
                description: "hook string: grep 800 text files, recursive literal search",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_grep]),
                original_program: "/usr/bin/grep".to_string(),
                original_args: strings(["-R", "NEEDLE", &path_string(&grep_root)]),
                stdin_file: None,
            },
        ]
    }
}

fn write_repeated(path: &Path, chunk: &[u8], count: usize) -> Result<()> {
    let mut file = fs::File::create(path)?;
    for _ in 0..count {
        file.write_all(chunk)?;
    }
    Ok(())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn strings<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(ToString::to_string).collect()
}
// CODEGEN-END
