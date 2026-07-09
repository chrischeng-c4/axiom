// SPEC-MANAGED: apps/cap/tech-design/semantic/source/projects-cap-benches-command_resources-rs.md#rust-source-unit
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
    CpuWin,
    RssFallback,
    Takeover,
    Candidate,
}

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#e2e-test
impl Gate {
    fn label(self) -> &'static str {
        match self {
            Self::DualWin => "dual-win",
            Self::CpuWin => "cpu-win",
            Self::RssFallback => "rss-fallback",
            Self::Takeover => "takeover",
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
            Self::CpuWin if cpu_ratio >= 1.0 => Some("cpu-win requires CPU below original"),
            Self::RssFallback if rss_ratio >= 1.0 => {
                Some("rss-fallback requires RSS below original")
            }
            Self::Takeover | Self::Candidate | Self::DualWin | Self::CpuWin | Self::RssFallback => {
                None
            }
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

/// @spec apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#e2e-test
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
        let small_list_dir = root.join("ls-small");
        fs::create_dir(&small_list_dir)?;
        for idx in 0..8 {
            fs::write(small_list_dir.join(format!("tiny-{idx}.txt")), b"x")?;
        }
        fs::write(small_list_dir.join(".hidden-alpha"), b"x")?;
        fs::write(small_list_dir.join(".hidden-beta"), b"x")?;

        let cat_file = root.join("cat-large.txt");
        write_repeated(&cat_file, b"0123456789abcdef\n", 512 * 1024)?;

        let tr_input = root.join("tr-input.txt");
        write_repeated(&tr_input, b"abcdefghijklmnopqrstuvwxyz\n", 320 * 1024)?;

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
        let small_sed_file = root.join("sed-small.txt");
        fs::write(&small_sed_file, "one\ntwo\nthree\nfour\n")?;

        let sort_file = root.join("sort-lines.txt");
        let mut sort = fs::File::create(&sort_file)?;
        for idx in (0..500_000).rev() {
            writeln!(sort, "line-{idx:06}")?;
        }
        let small_sort_file = root.join("sort-small.txt");
        fs::write(&small_sort_file, "c\na\nb\n")?;

        let cut_file = root.join("cut-lines.csv");
        let mut cut = fs::File::create(&cut_file)?;
        for idx in 0..200_000 {
            if idx % 1000 == 0 {
                writeln!(cut, "plain-{idx:06}")?;
            } else {
                writeln!(cut, "field-{idx:06},value-{idx:06},tail-{idx:06}")?;
            }
        }

        let find_root = root.join("find-tree");
        for dir_idx in 0..80 {
            let subdir = find_root.join(format!("dir-{dir_idx:03}"));
            fs::create_dir_all(&subdir)?;
            for file_idx in 0..20 {
                fs::write(subdir.join(format!("item-{file_idx:03}.txt")), b"find\n")?;
                fs::write(subdir.join(format!("item-{file_idx:03}.rs")), b"find\n")?;
                fs::write(subdir.join(format!("item-{file_idx:03}.bin")), b"find\n")?;
            }
        }
        let small_find_root = root.join("find-small");
        fs::create_dir_all(&small_find_root)?;
        for idx in 0..8 {
            fs::write(small_find_root.join(format!("small-{idx}.txt")), b"find\n")?;
        }
        let small_find_nested = small_find_root.join("nested");
        fs::create_dir_all(&small_find_nested)?;
        fs::write(small_find_nested.join("nested-small.txt"), b"find\n")?;
        fs::write(small_find_nested.join("nested-small.rs"), b"find\n")?;

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
        let small_grep_root = root.join("grep-small");
        fs::create_dir_all(&small_grep_root)?;
        for idx in 0..8 {
            fs::write(
                small_grep_root.join(format!("small-{idx}.txt")),
                "ordinary searchable text\nNEEDLE once\n",
            )?;
        }
        let wc_root = root.join("wc-many");
        fs::create_dir(&wc_root)?;
        for idx in 0..2_000 {
            fs::write(
                wc_root.join(format!("count-{idx:04}.txt")),
                "one\ntwo\nthree\nfour\n",
            )?;
        }
        let xargs_wc_input = root.join("xargs-wc-input.txt");
        let mut xargs_wc = fs::File::create(&xargs_wc_input)?;
        let awk_xargs_wc_input = root.join("awk-xargs-wc-input.txt");
        let mut awk_xargs_wc = fs::File::create(&awk_xargs_wc_input)?;
        for idx in 0..2_000 {
            let path = path_string(&wc_root.join(format!("count-{idx:04}.txt")));
            writeln!(xargs_wc, "{path}")?;
            writeln!(awk_xargs_wc, "{path} NEEDLE")?;
        }

        let xargs_input = root.join("xargs-input.txt");
        let mut xargs = fs::File::create(&xargs_input)?;
        for idx in 0..20_000 {
            writeln!(xargs, "item-{idx:05}")?;
        }
        let xargs_pipe_file = root.join("xargs-pipe-lines.txt");
        let mut xargs_pipe = fs::File::create(&xargs_pipe_file)?;
        for idx in (0..5_000).rev() {
            writeln!(xargs_pipe, "pipe-token-{idx:05}")?;
        }

        Ok(Self { _dir: dir, root })
    }

    fn scenarios(&self) -> Vec<Scenario> {
        let list_dir = self.root.join("ls-many");
        let small_list_dir = self.root.join("ls-small");
        let cat_file = self.root.join("cat-large.txt");
        let tr_input = self.root.join("tr-input.txt");
        let mkdir_existing = self.root.join("mkdir-existing/a/b");
        let touch_file = self.root.join("touch-existing.txt");
        let byte_window_file = self.root.join("byte-window.bin");
        let sed_file = self.root.join("sed-lines.txt");
        let small_sed_file = self.root.join("sed-small.txt");
        let sort_file = self.root.join("sort-lines.txt");
        let small_sort_file = self.root.join("sort-small.txt");
        let cut_file = self.root.join("cut-lines.csv");
        let find_root = self.root.join("find-tree");
        let small_find_root = self.root.join("find-small");
        let grep_root = self.root.join("grep-tree");
        let small_grep_root = self.root.join("grep-small");
        let wc_files = (0..2_000)
            .map(|idx| {
                path_string(
                    &self
                        .root
                        .join("wc-many")
                        .join(format!("count-{idx:04}.txt")),
                )
            })
            .collect::<Vec<_>>();
        let xargs_input = self.root.join("xargs-input.txt");
        let xargs_wc_input = self.root.join("xargs-wc-input.txt");
        let xargs_pipe_file = self.root.join("xargs-pipe-lines.txt");
        let long_basename_suffix = "suffix".repeat(78);
        let long_basename_path = format!(
            "/tmp/cap/bench/{}file{}",
            "nested/".repeat(78),
            long_basename_suffix
        );
        let long_dirname_path = format!("/tmp/cap/bench/{}file.txt", "nested/".repeat(140));
        let echo_words = (0..2_000)
            .map(|idx| format!("echo-word-{idx:04}"))
            .collect::<Vec<_>>();
        let printf_words = (0..2_000)
            .map(|idx| {
                if idx % 100 == 0 {
                    format!("printf-NEEDLE-{idx:04}")
                } else {
                    format!("printf-word-{idx:04}")
                }
            })
            .collect::<Vec<_>>();
        let cat_wc_pipe = format!("cat {} | wc -l", path_string(&cat_file));
        let cat_wc_bytes_pipe = format!("cat {} | wc -c", path_string(&cat_file));
        let echo_wc_pipe = format!("echo {} | wc -l", echo_words.join(" "));
        let echo_wc_words_pipe = format!("echo {} | wc -w", echo_words.join(" "));
        let echo_head_pipe = format!("echo -n {} | head -n 1", echo_words.join(" "));
        let echo_tail_pipe = format!("echo -n {} | tail -n 1", echo_words.join(" "));
        let echo_tr_pipe = format!("echo {} | tr a-z A-Z", echo_words.join(" "));
        let echo_awk_xargs_pipe = format!(
            "echo {} | awk '{{ print $1 }}' | xargs",
            echo_words.join(" ")
        );
        let echo_xargs_echo_pipe = format!("echo {} | xargs echo", echo_words.join(" "));
        let echo_xargs_wc_pipe = format!("echo {} | xargs wc -l", wc_files.join(" "));
        let xargs_stdin_wc_pipe = "xargs echo | wc -l".to_string();
        let xargs_n1_stdin_wc_pipe = "xargs -n 1 echo | wc -l".to_string();
        let xargs_stdin_grep_wc_pipe = "xargs echo | grep item-19999 | wc -l".to_string();
        let grep_stdin_wc_pipe = "grep NEEDLE | wc -l".to_string();
        let printf_wc_pipe = format!("printf '%s\\n' {} | wc -l", printf_words.join(" "));
        let printf_wc_bytes_pipe = format!("printf '%s\\n' {} | wc -c", printf_words.join(" "));
        let printf_head_pipe = format!("printf '%s\\n' {} | head -n 50", printf_words.join(" "));
        let printf_tail_pipe = format!("printf '%s\\n' {} | tail -n 50", printf_words.join(" "));
        let printf_awk_wc_pipe = format!(
            "printf '%s\\n' {} | awk '{{ print $1 }}' | wc -l",
            printf_words.join(" ")
        );
        let printf_awk_sort_uniq_pipe = format!(
            "printf '%s\\n' {} | awk '{{ print $1 }}' | sort | uniq",
            printf_words.join(" ")
        );
        let printf_grep_pipe = format!("printf '%s\\n' {} | grep NEEDLE", printf_words.join(" "));
        let printf_grep_wc_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | wc -l",
            printf_words.join(" ")
        );
        let printf_grep_head_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | head -n 50",
            printf_words.join(" ")
        );
        let printf_grep_tail_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | tail -n 50",
            printf_words.join(" ")
        );
        let printf_grep_sort_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | sort",
            printf_words.join(" ")
        );
        let printf_grep_sort_uniq_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | sort | uniq",
            printf_words.join(" ")
        );
        let printf_grep_sort_uniq_wc_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | sort | uniq | wc -l",
            printf_words.join(" ")
        );
        let printf_grep_sort_uniq_wc_words_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | sort | uniq | wc -w",
            printf_words.join(" ")
        );
        let printf_grep_sort_uniq_head_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | sort | uniq | head -n 50",
            printf_words.join(" ")
        );
        let printf_grep_sort_uniq_xargs_wc_pipe = format!(
            "printf '%s\\n' {} | grep count-19 | sort | uniq | xargs wc -l",
            wc_files.join(" ")
        );
        let printf_grep_sort_wc_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | sort | wc -l",
            printf_words.join(" ")
        );
        let printf_grep_sort_head_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | sort | head -n 50",
            printf_words.join(" ")
        );
        let printf_grep_sort_tail_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | sort | tail -n 50",
            printf_words.join(" ")
        );
        let printf_grep_sort_xargs_echo_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | sort | xargs echo",
            printf_words.join(" ")
        );
        let printf_grep_xargs_echo_pipe = format!(
            "printf '%s\\n' {} | grep NEEDLE | xargs echo",
            printf_words.join(" ")
        );
        let printf_tr_pipe = format!("printf '%s\\n' {} | tr a-z A-Z", printf_words.join(" "));
        let printf_sort_pipe = format!("printf '%s\\n' {} | sort", printf_words.join(" "));
        let printf_sort_uniq_pipe =
            format!("printf '%s\\n' {} | sort | uniq", printf_words.join(" "));
        let printf_sort_uniq_wc_pipe = format!(
            "printf '%s\\n' {} | sort | uniq | wc -l",
            printf_words.join(" ")
        );
        let printf_sort_uniq_head_pipe = format!(
            "printf '%s\\n' {} | sort | uniq | head -n 50",
            printf_words.join(" ")
        );
        let printf_sort_uniq_xargs_wc_pipe = format!(
            "printf '%s\\n' {} | sort | uniq | xargs wc -l",
            wc_files.join(" ")
        );
        let printf_sort_wc_pipe =
            format!("printf '%s\\n' {} | sort | wc -l", printf_words.join(" "));
        let printf_sort_head_pipe = format!(
            "printf '%s\\n' {} | sort | head -n 50",
            printf_words.join(" ")
        );
        let printf_sort_tail_pipe = format!(
            "printf '%s\\n' {} | sort | tail -n 50",
            printf_words.join(" ")
        );
        let printf_sort_xargs_echo_pipe = format!(
            "printf '%s\\n' {} | sort | xargs echo",
            printf_words.join(" ")
        );
        let printf_sort_xargs_wc_pipe =
            format!("printf '%s\\n' {} | sort | xargs wc -l", wc_files.join(" "));
        let printf_xargs_echo_pipe =
            format!("printf '%s\\n' {} | xargs echo", printf_words.join(" "));
        let printf_xargs_wc_pipe = format!("printf '%s\\n' {} | xargs wc -l", wc_files.join(" "));
        let seq_wc_pipe = "seq 1 200000 | wc -l".to_string();
        let seq_head_pipe = "seq 1 200000 | head -n 50".to_string();
        let seq_tail_pipe = "seq 1 200000 | tail -n 50".to_string();
        let seq_sort_pipe = "seq 1 200000 | sort".to_string();
        let seq_sort_uniq_pipe = "seq 1 200000 | sort | uniq".to_string();
        let seq_sort_uniq_wc_pipe = "seq 1 200000 | sort | uniq | wc -l".to_string();
        let seq_sort_uniq_head_pipe = "seq 1 200000 | sort | uniq | head -n 50".to_string();
        let seq_sort_uniq_sort_xargs_echo_pipe =
            "seq 1 5000 | sort | uniq | sort | xargs echo".to_string();
        let seq_sort_wc_pipe = "seq 1 200000 | sort | wc -l".to_string();
        let seq_sort_head_pipe = "seq 1 200000 | sort | head -n 50".to_string();
        let seq_sort_tail_pipe = "seq 1 200000 | sort | tail -n 50".to_string();
        let seq_sort_xargs_echo_pipe = "seq 1 5000 | sort | xargs echo".to_string();
        let seq_grep_pipe = "seq 1 200000 | grep 199".to_string();
        let seq_grep_wc_pipe = "seq 1 200000 | grep 199 | wc -l".to_string();
        let seq_grep_head_pipe = "seq 1 200000 | grep 199 | head -n 50".to_string();
        let seq_grep_tail_pipe = "seq 1 200000 | grep 199 | tail -n 50".to_string();
        let seq_grep_sort_pipe = "seq 1 200000 | grep 199 | sort".to_string();
        let seq_grep_sort_uniq_pipe = "seq 1 200000 | grep 199 | sort | uniq".to_string();
        let seq_grep_sort_uniq_wc_pipe =
            "seq 1 200000 | grep 199 | sort | uniq | wc -l".to_string();
        let seq_grep_sort_uniq_head_pipe =
            "seq 1 200000 | grep 199 | sort | uniq | head -n 50".to_string();
        let seq_grep_sort_uniq_sort_xargs_echo_pipe =
            "seq 1 5000 | grep 199 | sort | uniq | sort | xargs echo".to_string();
        let seq_grep_sort_wc_pipe = "seq 1 200000 | grep 199 | sort | wc -l".to_string();
        let seq_grep_sort_head_pipe = "seq 1 200000 | grep 199 | sort | head -n 50".to_string();
        let seq_grep_sort_tail_pipe = "seq 1 200000 | grep 199 | sort | tail -n 50".to_string();
        let seq_grep_sort_xargs_echo_pipe = "seq 1 5000 | grep 199 | sort | xargs echo".to_string();
        let seq_grep_xargs_echo_pipe = "seq 1 5000 | grep 199 | xargs echo".to_string();
        let seq_xargs_echo_pipe = "seq 1 5000 | xargs echo".to_string();
        let yes_head_pipe = "yes READY | head -n 20000".to_string();
        let which_wc_pipe = "which sh echo | wc -l".to_string();
        let which_head_pipe = "which sh echo | head -n 1".to_string();
        let which_tail_pipe = "which sh echo | tail -n 1".to_string();
        let which_grep_wc_pipe = "which sh echo | grep / | wc -l".to_string();
        let which_xargs_pipe = "which sh echo | xargs echo".to_string();
        let which_sort_wc_pipe = "which sh echo | sort | wc -l".to_string();
        let which_sort_xargs_pipe = "which sh echo | sort | xargs echo".to_string();
        let which_all_wc_pipe = "which -a sh echo | wc -l".to_string();
        let which_all_xargs_pipe = "which -a sh echo | xargs echo".to_string();
        let which_all_sort_xargs_pipe = "which -a sh echo | sort | xargs echo".to_string();
        let command_v_wc_pipe = "command -v sh echo | wc -l".to_string();
        let command_v_head_pipe = "command -v sh echo | head -n 1".to_string();
        let command_v_tail_pipe = "command -v sh echo | tail -n 1".to_string();
        let command_v_grep_wc_pipe = "command -v sh echo | grep / | wc -l".to_string();
        let command_v_xargs_pipe = "command -v sh echo | xargs echo".to_string();
        let command_v_sort_wc_pipe = "command -v sh echo | sort | wc -l".to_string();
        let command_v_sort_xargs_pipe = "command -v sh echo | sort | xargs echo".to_string();
        let printenv_path_wc_pipe = "printenv PATH | wc -l".to_string();
        let printenv_path_grep_pipe = "printenv PATH | grep /".to_string();
        let printenv_path_grep_wc_pipe = "printenv PATH | grep / | wc -l".to_string();
        let printenv_path_xargs_pipe = "printenv PATH | xargs echo".to_string();
        let printenv_path_sort_xargs_pipe = "printenv PATH | sort | xargs echo".to_string();
        let true_wc_pipe = "true | wc -l".to_string();
        let false_wc_pipe = "false | wc -l".to_string();
        let false_grep_wc_pipe = "false | grep NEEDLE | wc -l".to_string();
        let true_xargs_echo_pipe = "true | xargs echo".to_string();
        let mkdir_pipe_dir = self.root.join("mkdir-pipe-created");
        let mkdir_wc_pipe = format!("mkdir -p {} | wc -l", path_string(&mkdir_pipe_dir));
        let mkdir_xargs_echo_pipe =
            format!("mkdir -p {} | xargs echo", path_string(&mkdir_pipe_dir));
        let touch_pipe_file = self.root.join("touch-pipe-created.txt");
        let touch_wc_pipe = format!("touch {} | wc -l", path_string(&touch_pipe_file));
        let touch_sort_xargs_echo_pipe = format!(
            "touch {} | sort | xargs echo",
            path_string(&touch_pipe_file)
        );
        let test_missing = self.root.join("test-missing");
        let test_wc_pipe = format!("test -f {} | wc -l", path_string(&cat_file));
        let test_xargs_echo_pipe = format!("test ! -e {} | xargs echo", path_string(&test_missing));
        let bracket_sort_xargs_echo_pipe =
            format!("[ -d {} ] | sort | xargs echo", path_string(&find_root));
        let test_grep_wc_pipe = format!("test -d {} | grep NEEDLE | wc -l", path_string(&cat_file));
        let wc_xargs_echo_pipe = format!("wc -l {} | xargs echo", wc_files[0]);
        let wc_multi_wc_pipe = format!("wc -c {} {} | wc -l", wc_files[0], wc_files[1]);
        let wc_grep_wc_pipe = format!("wc -l {} {} | grep total | wc -l", wc_files[0], wc_files[1]);
        let wc_sort_xargs_echo_pipe = format!("wc -w {} | sort | xargs echo", wc_files[1]);
        let wc_stdin_wc_pipe = "wc -l | wc -l".to_string();
        let wc_stdin_grep_wc_pipe = "wc -w | grep 500000 | wc -l".to_string();
        let wc_stdin_sort_xargs_echo_pipe = "wc -l | sort | xargs echo".to_string();
        let printf_literal_wc_pipe = "printf 'alpha\\nbeta\\n' | wc -l".to_string();
        let printf_literal_grep_wc_pipe =
            "printf 'alpha\\nbeta\\n' | grep beta | wc -l".to_string();
        let printf_literal_sort_xargs_echo_pipe =
            "printf 'zeta\\nalpha\\n' | sort | xargs echo".to_string();
        let du_wc_pipe = format!("du -sk {} | wc -l", path_string(&find_root));
        let du_xargs_echo_pipe = format!("du -sk {} | xargs echo", path_string(&find_root));
        let du_grep_wc_pipe = format!(
            "du -sk {} | grep find-tree | wc -l",
            path_string(&find_root)
        );
        let hostname_wc_pipe = "hostname | wc -l".to_string();
        let hostname_head_pipe = "hostname | head -n 1".to_string();
        let hostname_tail_pipe = "hostname | tail -n 1".to_string();
        let hostname_pattern = Command::new("hostname")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|hostname| {
                hostname
                    .trim()
                    .chars()
                    .find(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
                    .map(|ch| ch.to_string())
            })
            .unwrap_or_else(|| "localhost".to_string());
        let hostname_grep_wc_pipe = format!("hostname | grep {hostname_pattern} | wc -l");
        let hostname_sort_pipe = "hostname | sort".to_string();
        let hostname_xargs_pipe = "hostname | xargs echo".to_string();
        let hostname_sort_xargs_pipe = "hostname | sort | xargs echo".to_string();
        let ls_wc_pipe = format!("ls -1 {} | wc -l", path_string(&list_dir));
        let ls_head_pipe = format!("ls -1 {} | head -n 50", path_string(&list_dir));
        let ls_tail_pipe = format!("ls -1 {} | tail -n 50", path_string(&list_dir));
        let ls_sort_pipe = format!("ls -1 {} | sort", path_string(&list_dir));
        let ls_sort_uniq_pipe = format!("ls -1 {} | sort | uniq", path_string(&list_dir));
        let ls_sort_uniq_wc_pipe =
            format!("ls -1 {} | sort | uniq | wc -l", path_string(&list_dir));
        let ls_sort_wc_pipe = format!("ls -1 {} | sort | wc -l", path_string(&list_dir));
        let ls_sort_head_pipe = format!("ls -1 {} | sort | head -n 50", path_string(&list_dir));
        let ls_sort_tail_pipe = format!("ls -1 {} | sort | tail -n 50", path_string(&list_dir));
        let ls_sort_xargs_echo_pipe =
            format!("ls -1 {} | sort | xargs echo", path_string(&small_list_dir));
        let ls_grep_pipe = format!("ls -1 {} | grep file-19", path_string(&list_dir));
        let ls_grep_wc_pipe = format!("ls -1 {} | grep file-19 | wc -l", path_string(&list_dir));
        let ls_grep_head_pipe = format!(
            "ls -1 {} | grep file-19 | head -n 50",
            path_string(&list_dir)
        );
        let ls_grep_tail_pipe = format!(
            "ls -1 {} | grep file-19 | tail -n 50",
            path_string(&list_dir)
        );
        let ls_grep_sort_pipe = format!("ls -1 {} | grep file-19 | sort", path_string(&list_dir));
        let ls_grep_sort_uniq_wc_pipe = format!(
            "ls -1 {} | grep file-19 | sort | uniq | wc -l",
            path_string(&list_dir)
        );
        let ls_grep_xargs_echo_pipe = format!(
            "ls -1 {} | grep file-19 | xargs echo",
            path_string(&list_dir)
        );
        let ls_grep_sort_xargs_echo_pipe = format!(
            "ls -1 {} | grep file-19 | sort | xargs echo",
            path_string(&list_dir)
        );
        let ls_xargs_echo_pipe = format!("ls -1 {} | xargs echo", path_string(&list_dir));
        let ls_all_wc_pipe = format!("ls -a {} | wc -l", path_string(&small_list_dir));
        let ls_all_grep_wc_pipe = format!(
            "ls -a {} | grep hidden | wc -l",
            path_string(&small_list_dir)
        );
        let ls_all_sort_tail_pipe =
            format!("ls -a {} | sort | tail -n 1", path_string(&small_list_dir));
        let ls_all_xargs_echo_pipe = format!("ls -a {} | xargs echo", path_string(&small_list_dir));
        let ls_all_sort_xargs_echo_pipe =
            format!("ls -a {} | sort | xargs echo", path_string(&small_list_dir));
        let ls_almost_wc_pipe = format!("ls -A {} | wc -l", path_string(&small_list_dir));
        let ls_almost_grep_wc_pipe = format!(
            "ls -A {} | grep hidden | wc -l",
            path_string(&small_list_dir)
        );
        let ls_almost_sort_tail_pipe =
            format!("ls -A {} | sort | tail -n 1", path_string(&small_list_dir));
        let ls_almost_xargs_echo_pipe =
            format!("ls -A {} | xargs echo", path_string(&small_list_dir));
        let ls_almost_sort_xargs_echo_pipe =
            format!("ls -A {} | sort | xargs echo", path_string(&small_list_dir));
        let sort_uniq_pipe = format!("sort {} | uniq", path_string(&sort_file));
        let sort_uniq_wc_pipe = format!("sort {} | uniq | wc -l", path_string(&sort_file));
        let sort_uniq_wc_bytes_pipe = format!("sort {} | uniq | wc -c", path_string(&sort_file));
        let sort_grep_pipe = format!("sort {} | grep line-19", path_string(&sort_file));
        let sort_grep_wc_pipe = format!("sort {} | grep line-19 | wc -l", path_string(&sort_file));
        let sort_grep_xargs_wc_pipe = format!(
            "sort {} | grep count-19 | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let sort_head_pipe = format!("sort {} | head -n 50", path_string(&sort_file));
        let sort_tail_pipe = format!("sort {} | tail -n 50", path_string(&sort_file));
        let sort_wc_pipe = format!("sort {} | wc -l", path_string(&sort_file));
        let sort_wc_words_pipe = format!("sort {} | wc -w", path_string(&sort_file));
        let head_wc_pipe = format!("head -n 50000 {} | wc -l", path_string(&sort_file));
        let head_stdin_wc_pipe = "head -n 50000 | wc -l".to_string();
        let head_head_pipe = format!("head -n 50000 {} | head -n 50", path_string(&sort_file));
        let head_tail_pipe = format!("head -n 50000 {} | tail -n 50", path_string(&sort_file));
        let head_sort_pipe = format!("head -n 50000 {} | sort", path_string(&sort_file));
        let head_sort_uniq_pipe =
            format!("head -n 50000 {} | sort | uniq", path_string(&sort_file));
        let head_sort_uniq_wc_pipe = format!(
            "head -n 50000 {} | sort | uniq | wc -l",
            path_string(&sort_file)
        );
        let head_sort_wc_pipe = format!("head -n 50000 {} | sort | wc -l", path_string(&sort_file));
        let head_sort_head_pipe = format!(
            "head -n 50000 {} | sort | head -n 50",
            path_string(&sort_file)
        );
        let head_sort_tail_pipe = format!(
            "head -n 50000 {} | sort | tail -n 50",
            path_string(&sort_file)
        );
        let head_xargs_echo_pipe =
            format!("head -n 50000 {} | xargs echo", path_string(&sort_file));
        let head_xargs_wc_pipe =
            format!("head -n 500 {} | xargs wc -l", path_string(&xargs_wc_input));
        let head_sort_xargs_echo_pipe = format!(
            "head -n 50000 {} | sort | xargs echo",
            path_string(&sort_file)
        );
        let head_sort_xargs_wc_pipe = format!(
            "head -n 500 {} | sort | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let head_grep_pipe = format!("head -n 50000 {} | grep 499", path_string(&sort_file));
        let head_grep_wc_pipe = format!(
            "head -n 50000 {} | grep 499 | wc -l",
            path_string(&sort_file)
        );
        let head_grep_head_pipe = format!(
            "head -n 50000 {} | grep 499 | head -n 50",
            path_string(&sort_file)
        );
        let head_grep_tail_pipe = format!(
            "head -n 50000 {} | grep 499 | tail -n 50",
            path_string(&sort_file)
        );
        let head_grep_sort_pipe = format!(
            "head -n 50000 {} | grep 499 | sort",
            path_string(&sort_file)
        );
        let head_grep_sort_uniq_pipe = format!(
            "head -n 50000 {} | grep 499 | sort | uniq",
            path_string(&sort_file)
        );
        let head_grep_sort_uniq_wc_pipe = format!(
            "head -n 50000 {} | grep 499 | sort | uniq | wc -l",
            path_string(&sort_file)
        );
        let head_grep_sort_wc_pipe = format!(
            "head -n 50000 {} | grep 499 | sort | wc -l",
            path_string(&sort_file)
        );
        let head_grep_sort_head_pipe = format!(
            "head -n 50000 {} | grep 499 | sort | head -n 50",
            path_string(&sort_file)
        );
        let head_grep_sort_tail_pipe = format!(
            "head -n 50000 {} | grep 499 | sort | tail -n 50",
            path_string(&sort_file)
        );
        let head_grep_xargs_echo_pipe = format!(
            "head -n 50000 {} | grep 499 | xargs echo",
            path_string(&sort_file)
        );
        let head_grep_xargs_wc_pipe = format!(
            "head -n 500 {} | grep count-0 | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let head_grep_sort_xargs_echo_pipe = format!(
            "head -n 50000 {} | grep 499 | sort | xargs echo",
            path_string(&sort_file)
        );
        let head_grep_sort_xargs_wc_pipe = format!(
            "head -n 500 {} | grep count-0 | sort | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let tail_wc_pipe = format!("tail -n 50000 {} | wc -l", path_string(&sort_file));
        let tail_stdin_wc_pipe = "tail -n 50000 | wc -l".to_string();
        let tail_head_pipe = format!("tail -n 50000 {} | head -n 50", path_string(&sort_file));
        let tail_tail_pipe = format!("tail -n 50000 {} | tail -n 50", path_string(&sort_file));
        let tail_sort_pipe = format!("tail -n 50000 {} | sort", path_string(&sort_file));
        let tail_sort_uniq_pipe =
            format!("tail -n 50000 {} | sort | uniq", path_string(&sort_file));
        let tail_sort_uniq_wc_pipe = format!(
            "tail -n 50000 {} | sort | uniq | wc -l",
            path_string(&sort_file)
        );
        let tail_sort_wc_pipe = format!("tail -n 50000 {} | sort | wc -l", path_string(&sort_file));
        let tail_sort_head_pipe = format!(
            "tail -n 50000 {} | sort | head -n 50",
            path_string(&sort_file)
        );
        let tail_sort_tail_pipe = format!(
            "tail -n 50000 {} | sort | tail -n 50",
            path_string(&sort_file)
        );
        let tail_xargs_echo_pipe =
            format!("tail -n 50000 {} | xargs echo", path_string(&sort_file));
        let tail_xargs_wc_pipe =
            format!("tail -n 500 {} | xargs wc -l", path_string(&xargs_wc_input));
        let tail_sort_xargs_echo_pipe = format!(
            "tail -n 50000 {} | sort | xargs echo",
            path_string(&sort_file)
        );
        let tail_sort_xargs_wc_pipe = format!(
            "tail -n 500 {} | sort | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let tail_grep_pipe = format!("tail -n 50000 {} | grep 049", path_string(&sort_file));
        let tail_grep_wc_pipe = format!(
            "tail -n 50000 {} | grep 049 | wc -l",
            path_string(&sort_file)
        );
        let tail_grep_head_pipe = format!(
            "tail -n 50000 {} | grep 049 | head -n 50",
            path_string(&sort_file)
        );
        let tail_grep_tail_pipe = format!(
            "tail -n 50000 {} | grep 049 | tail -n 50",
            path_string(&sort_file)
        );
        let tail_grep_sort_pipe = format!(
            "tail -n 50000 {} | grep 049 | sort",
            path_string(&sort_file)
        );
        let tail_grep_sort_uniq_pipe = format!(
            "tail -n 50000 {} | grep 049 | sort | uniq",
            path_string(&sort_file)
        );
        let tail_grep_sort_uniq_wc_pipe = format!(
            "tail -n 50000 {} | grep 049 | sort | uniq | wc -l",
            path_string(&sort_file)
        );
        let tail_grep_sort_wc_pipe = format!(
            "tail -n 50000 {} | grep 049 | sort | wc -l",
            path_string(&sort_file)
        );
        let tail_grep_sort_head_pipe = format!(
            "tail -n 50000 {} | grep 049 | sort | head -n 50",
            path_string(&sort_file)
        );
        let tail_grep_sort_tail_pipe = format!(
            "tail -n 50000 {} | grep 049 | sort | tail -n 50",
            path_string(&sort_file)
        );
        let tail_grep_xargs_echo_pipe = format!(
            "tail -n 50000 {} | grep 049 | xargs echo",
            path_string(&sort_file)
        );
        let tail_grep_xargs_wc_pipe = format!(
            "tail -n 500 {} | grep count-19 | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let tail_grep_sort_xargs_echo_pipe = format!(
            "tail -n 50000 {} | grep 049 | sort | xargs echo",
            path_string(&sort_file)
        );
        let tail_grep_sort_xargs_wc_pipe = format!(
            "tail -n 500 {} | grep count-19 | sort | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_head_pipe = format!("cat {} | head -n 50", path_string(&cat_file));
        let cat_tail_pipe = format!("cat {} | tail -n 50", path_string(&sed_file));
        let cat_head_wc_pipe = format!("cat {} | head -n 5000 | wc -l", path_string(&sed_file));
        let cat_tail_wc_pipe = format!("cat {} | tail -n 5000 | wc -l", path_string(&sed_file));
        let cat_head_sort_uniq_wc_pipe = format!(
            "cat {} | head -n 5000 | sort | uniq | wc -l",
            path_string(&sort_file)
        );
        let cat_tail_sort_uniq_wc_pipe = format!(
            "cat {} | tail -n 5000 | sort | uniq | wc -l",
            path_string(&sort_file)
        );
        let cat_head_grep_sort_xargs_echo_pipe = format!(
            "cat {} | head -n 50000 | grep 049 | sort | xargs echo",
            path_string(&sort_file)
        );
        let cat_tail_grep_sort_xargs_echo_pipe = format!(
            "cat {} | tail -n 50000 | grep 049 | sort | xargs echo",
            path_string(&sort_file)
        );
        let cat_head_xargs_wc_pipe = format!(
            "cat {} | head -n 500 | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_tail_xargs_wc_pipe = format!(
            "cat {} | tail -n 500 | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_grep_pipe = format!("cat {} | grep NEEDLE", path_string(&sed_file));
        let cat_grep_wc_pipe = format!("cat {} | grep NEEDLE | wc -l", path_string(&sed_file));
        let cat_grep_head_pipe =
            format!("cat {} | grep NEEDLE | head -n 50", path_string(&sed_file));
        let cat_grep_tail_pipe =
            format!("cat {} | grep NEEDLE | tail -n 50", path_string(&sed_file));
        let cat_grep_sort_pipe = format!("cat {} | grep NEEDLE | sort", path_string(&sed_file));
        let cat_grep_sort_uniq_pipe =
            format!("cat {} | grep NEEDLE | sort | uniq", path_string(&sed_file));
        let cat_grep_sort_uniq_wc_pipe = format!(
            "cat {} | grep NEEDLE | sort | uniq | wc -l",
            path_string(&sed_file)
        );
        let cat_grep_sort_uniq_head_pipe = format!(
            "cat {} | grep NEEDLE | sort | uniq | head -n 50",
            path_string(&sed_file)
        );
        let cat_grep_sort_uniq_tail_pipe = format!(
            "cat {} | grep NEEDLE | sort | uniq | tail -n 50",
            path_string(&sed_file)
        );
        let cat_grep_sort_uniq_sort_xargs_pipe = format!(
            "cat {} | grep NEEDLE | sort | uniq | sort | xargs echo",
            path_string(&sed_file)
        );
        let cat_grep_sort_uniq_xargs_wc_pipe = format!(
            "cat {} | grep count-19 | sort | uniq | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_grep_sort_uniq_sort_xargs_wc_pipe = format!(
            "cat {} | grep count-19 | sort | uniq | sort | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_grep_sort_wc_pipe = format!(
            "cat {} | grep NEEDLE | sort | wc -l",
            path_string(&sed_file)
        );
        let cat_grep_sort_head_pipe = format!(
            "cat {} | grep NEEDLE | sort | head -n 50",
            path_string(&sed_file)
        );
        let cat_grep_sort_tail_pipe = format!(
            "cat {} | grep NEEDLE | sort | tail -n 50",
            path_string(&sed_file)
        );
        let cat_cut_pipe = format!("cat {} | cut -d, -f1", path_string(&cut_file));
        let cat_tr_pipe = format!("cat {} | tr a-z A-Z", path_string(&tr_input));
        let cat_uniq_pipe = format!("cat {} | uniq", path_string(&sort_file));
        let cat_uniq_wc_pipe = format!("cat {} | uniq | wc -l", path_string(&sort_file));
        let uniq_wc_pipe = format!("uniq {} | wc -l", path_string(&sort_file));
        let uniq_grep_pipe = format!("uniq {} | grep line-19", path_string(&sort_file));
        let uniq_grep_wc_pipe = format!("uniq {} | grep line-19 | wc -l", path_string(&sort_file));
        let uniq_grep_xargs_wc_pipe = format!(
            "uniq {} | grep count-19 | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_sort_pipe = format!("cat {} | sort", path_string(&sort_file));
        let cat_sort_uniq_pipe = format!("cat {} | sort | uniq", path_string(&sort_file));
        let cat_sort_uniq_wc_pipe =
            format!("cat {} | sort | uniq | wc -l", path_string(&sort_file));
        let cat_sort_grep_pipe = format!("cat {} | sort | grep line-19", path_string(&sort_file));
        let cat_sort_grep_wc_pipe = format!(
            "cat {} | sort | grep line-19 | wc -l",
            path_string(&sort_file)
        );
        let cat_sort_grep_xargs_wc_pipe = format!(
            "cat {} | sort | grep count-19 | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_sort_wc_pipe = format!("cat {} | sort | wc -l", path_string(&sort_file));
        let cat_sort_head_pipe = format!("cat {} | sort | head -n 50", path_string(&sort_file));
        let cat_sort_tail_pipe = format!("cat {} | sort | tail -n 50", path_string(&sort_file));
        let cat_xargs_echo_pipe = format!("cat {} | xargs echo", path_string(&xargs_pipe_file));
        let sort_xargs_echo_pipe = format!("sort {} | xargs echo", path_string(&xargs_pipe_file));
        let cat_sort_xargs_echo_pipe =
            format!("cat {} | sort | xargs echo", path_string(&xargs_pipe_file));
        let cat_xargs_wc_pipe = format!("cat {} | xargs wc -l", path_string(&xargs_wc_input));
        let sort_xargs_wc_pipe = format!("sort {} | xargs wc -l", path_string(&xargs_wc_input));
        let cat_sort_xargs_wc_pipe =
            format!("cat {} | sort | xargs wc -l", path_string(&xargs_wc_input));
        let cat_grep_xargs_echo_pipe =
            format!("cat {} | grep NEEDLE | xargs echo", path_string(&sed_file));
        let cat_grep_xargs_wc_pipe = format!(
            "cat {} | grep count-19 | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_grep_sort_xargs_echo_pipe = format!(
            "cat {} | grep NEEDLE | sort | xargs echo",
            path_string(&sed_file)
        );
        let cat_grep_sort_xargs_wc_pipe = format!(
            "cat {} | grep count-19 | sort | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let grep_head_pipe = format!("grep -R NEEDLE {} | head -n 50", path_string(&grep_root));
        let grep_tail_pipe = format!("grep -R NEEDLE {} | tail -n 50", path_string(&grep_root));
        let grep_sort_pipe = format!("grep -R NEEDLE {} | sort", path_string(&grep_root));
        let grep_sort_uniq_pipe =
            format!("grep -R NEEDLE {} | sort | uniq", path_string(&grep_root));
        let grep_sort_uniq_wc_pipe = format!(
            "grep -R NEEDLE {} | sort | uniq | wc -l",
            path_string(&grep_root)
        );
        let grep_sort_uniq_head_pipe = format!(
            "grep -R NEEDLE {} | sort | uniq | head -n 50",
            path_string(&grep_root)
        );
        let grep_sort_uniq_tail_pipe = format!(
            "grep -R NEEDLE {} | sort | uniq | tail -n 50",
            path_string(&grep_root)
        );
        let grep_sort_uniq_sort_xargs_pipe = format!(
            "grep -R NEEDLE {} | sort | uniq | sort | xargs echo",
            path_string(&grep_root)
        );
        let grep_sort_wc_pipe =
            format!("grep -R NEEDLE {} | sort | wc -l", path_string(&grep_root));
        let grep_sort_head_pipe = format!(
            "grep -R NEEDLE {} | sort | head -n 50",
            path_string(&grep_root)
        );
        let grep_sort_tail_pipe = format!(
            "grep -R NEEDLE {} | sort | tail -n 50",
            path_string(&grep_root)
        );
        let grep_wc_pipe = format!("grep -R NEEDLE {} | wc -l", path_string(&grep_root));
        let grep_file_wc_pipe = format!("grep NEEDLE {} | wc -l", path_string(&sed_file));
        let grep_file_head_pipe = format!("grep NEEDLE {} | head -n 50", path_string(&sed_file));
        let grep_file_tail_pipe = format!("grep NEEDLE {} | tail -n 50", path_string(&sed_file));
        let grep_file_sort_pipe = format!("grep NEEDLE {} | sort", path_string(&sed_file));
        let grep_file_sort_uniq_pipe =
            format!("grep NEEDLE {} | sort | uniq", path_string(&sed_file));
        let grep_file_sort_uniq_wc_pipe = format!(
            "grep NEEDLE {} | sort | uniq | wc -l",
            path_string(&sed_file)
        );
        let grep_file_sort_uniq_head_pipe = format!(
            "grep NEEDLE {} | sort | uniq | head -n 50",
            path_string(&sed_file)
        );
        let grep_file_sort_uniq_tail_pipe = format!(
            "grep NEEDLE {} | sort | uniq | tail -n 50",
            path_string(&sed_file)
        );
        let grep_file_sort_uniq_sort_xargs_pipe = format!(
            "grep NEEDLE {} | sort | uniq | sort | xargs echo",
            path_string(&sed_file)
        );
        let grep_file_sort_uniq_xargs_wc_pipe = format!(
            "grep count- {} | sort | uniq | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let grep_file_sort_uniq_sort_xargs_wc_pipe = format!(
            "grep count- {} | sort | uniq | sort | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let grep_file_sort_wc_pipe =
            format!("grep NEEDLE {} | sort | wc -l", path_string(&sed_file));
        let grep_file_sort_head_pipe =
            format!("grep NEEDLE {} | sort | head -n 50", path_string(&sed_file));
        let grep_file_sort_tail_pipe =
            format!("grep NEEDLE {} | sort | tail -n 50", path_string(&sed_file));
        let grep_file_xargs_echo_pipe =
            format!("grep NEEDLE {} | xargs echo", path_string(&sed_file));
        let grep_file_xargs_wc_pipe =
            format!("grep count- {} | xargs wc -l", path_string(&xargs_wc_input));
        let grep_file_sort_xargs_echo_pipe =
            format!("grep NEEDLE {} | sort | xargs echo", path_string(&sed_file));
        let grep_file_sort_xargs_wc_pipe = format!(
            "grep count- {} | sort | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let grep_file_cut_pipe = format!("grep NEEDLE {} | cut -d ' ' -f1", path_string(&sed_file));
        let grep_file_cut_wc_pipe = format!(
            "grep NEEDLE {} | cut -d ' ' -f1 | wc -l",
            path_string(&sed_file)
        );
        let grep_file_cut_sort_uniq_wc_pipe = format!(
            "grep NEEDLE {} | cut -d ' ' -f1 | sort | uniq | wc -l",
            path_string(&sed_file)
        );
        let grep_file_cut_xargs_echo_pipe = format!(
            "grep NEEDLE {} | cut -d ' ' -f1 | xargs echo",
            path_string(&sed_file)
        );
        let grep_file_cut_xargs_wc_pipe = format!(
            "grep count- {} | cut -d ' ' -f1 | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let grep_file_awk_pipe = format!(
            "grep NEEDLE {} | awk '{{ print $1 }}'",
            path_string(&sed_file)
        );
        let grep_file_awk_wc_pipe = format!(
            "grep NEEDLE {} | awk '{{ print $1 }}' | wc -l",
            path_string(&sed_file)
        );
        let grep_file_awk_sort_uniq_wc_pipe = format!(
            "grep NEEDLE {} | awk '{{ print $1 }}' | sort | uniq | wc -l",
            path_string(&sed_file)
        );
        let grep_file_awk_xargs_echo_pipe = format!(
            "grep NEEDLE {} | awk '{{ print $1 }}' | xargs echo",
            path_string(&sed_file)
        );
        let grep_file_awk_xargs_wc_pipe = format!(
            "grep count- {} | awk '{{ print $1 }}' | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let awk_first_wc_pipe = format!("awk '{{ print $1 }}' {} | wc -l", path_string(&sed_file));
        let awk_stdin_wc_pipe = "awk '{ print $1 }' | wc -l".to_string();
        let awk_first_sort_uniq_wc_pipe = format!(
            "awk '{{ print $1 }}' {} | sort | uniq | wc -l",
            path_string(&sed_file)
        );
        let awk_first_xargs_echo_pipe = format!(
            "awk '{{ print $1 }}' {} | xargs echo",
            path_string(&sed_file)
        );
        let awk_first_xargs_wc_pipe = format!(
            "awk '{{ print $1 }}' {} | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let awk_first_grep_wc_pipe = format!(
            "awk '{{ print $1 }}' {} | grep line | wc -l",
            path_string(&sed_file)
        );
        let awk_first_grep_sort_uniq_wc_pipe = format!(
            "awk '{{ print $1 }}' {} | grep line | sort | uniq | wc -l",
            path_string(&sed_file)
        );
        let awk_first_grep_xargs_wc_pipe = format!(
            "awk '{{ print $1 }}' {} | grep count- | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let awk_xargs_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | xargs echo",
            path_string(&sed_file)
        );
        let awk_xargs_wc_input = self.root.join("awk-xargs-wc-input.txt");
        let awk_xargs_wc_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | xargs wc -l",
            path_string(&awk_xargs_wc_input)
        );
        let awk_wc_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | wc -l",
            path_string(&sed_file)
        );
        let awk_head_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | head -n 50",
            path_string(&sed_file)
        );
        let awk_tail_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | tail -n 50",
            path_string(&sed_file)
        );
        let awk_sort_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort",
            path_string(&sed_file)
        );
        let awk_sort_uniq_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq",
            path_string(&sed_file)
        );
        let awk_sort_uniq_wc_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | wc -l",
            path_string(&sed_file)
        );
        let awk_sort_uniq_head_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | head -n 50",
            path_string(&sed_file)
        );
        let awk_sort_uniq_sort_xargs_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | sort | xargs echo",
            path_string(&sed_file)
        );
        let awk_sort_uniq_xargs_wc_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | xargs wc -l",
            path_string(&awk_xargs_wc_input)
        );
        let awk_sort_uniq_sort_xargs_wc_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | sort | xargs wc -l",
            path_string(&awk_xargs_wc_input)
        );
        let awk_sort_wc_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | wc -l",
            path_string(&sed_file)
        );
        let awk_sort_head_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | head -n 50",
            path_string(&sed_file)
        );
        let awk_sort_tail_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | tail -n 50",
            path_string(&sed_file)
        );
        let awk_sort_xargs_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | xargs echo",
            path_string(&sed_file)
        );
        let awk_sort_xargs_wc_pipe = format!(
            "awk '/NEEDLE/ {{ print $1 }}' {} | sort | xargs wc -l",
            path_string(&awk_xargs_wc_input)
        );
        let cat_awk_pipe = format!(
            "cat {} | awk '/NEEDLE/ {{ print $1 }}'",
            path_string(&sed_file)
        );
        let cat_awk_wc_pipe = format!(
            "cat {} | awk '/NEEDLE/ {{ print $1 }}' | wc -l",
            path_string(&sed_file)
        );
        let cat_awk_head_pipe = format!(
            "cat {} | awk '/NEEDLE/ {{ print $1 }}' | head -n 50",
            path_string(&sed_file)
        );
        let cat_awk_sort_pipe = format!(
            "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort",
            path_string(&sed_file)
        );
        let cat_awk_sort_uniq_wc_pipe = format!(
            "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | uniq | wc -l",
            path_string(&sed_file)
        );
        let cat_awk_xargs_wc_pipe = format!(
            "cat {} | awk '/NEEDLE/ {{ print $1 }}' | xargs wc -l",
            path_string(&awk_xargs_wc_input)
        );
        let cat_awk_first_grep_tail_pipe = format!(
            "cat {} | awk '{{ print $1 }}' | grep line | tail -n 50",
            path_string(&sed_file)
        );
        let cat_awk_first_grep_sort_xargs_wc_pipe = format!(
            "cat {} | awk '{{ print $1 }}' | grep count- | sort | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_awk_sort_xargs_pipe = format!(
            "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | xargs echo",
            path_string(&sed_file)
        );
        let find_all_xargs_pipe = format!("find {} -type f | xargs wc -l", path_string(&find_root));
        let find_all_xargs_sort_pipe = format!(
            "find {} -type f | xargs wc -l | sort",
            path_string(&find_root)
        );
        let find_all_xargs_echo_pipe =
            format!("find {} -type f | xargs echo", path_string(&find_root));
        let find_all_xargs_default_pipe =
            format!("find {} -type f | xargs", path_string(&find_root));
        let find_all_wc_pipe = format!("find {} -type f | wc -l", path_string(&find_root));
        let find_all_head_pipe = format!("find {} -type f | head -n 50", path_string(&find_root));
        let find_all_tail_pipe = format!("find {} -type f | tail -n 50", path_string(&find_root));
        let find_all_sort_wc_pipe =
            format!("find {} -type f | sort | wc -l", path_string(&find_root));
        let find_maxdepth_wc_pipe = format!(
            "find {} -maxdepth 1 -type f | wc -l",
            path_string(&small_find_root)
        );
        let find_maxdepth_head_pipe = format!(
            "find {} -maxdepth 1 -type f | head -n 5",
            path_string(&small_find_root)
        );
        let find_maxdepth_grep_wc_pipe = format!(
            "find {} -maxdepth 1 -type f | grep small- | wc -l",
            path_string(&small_find_root)
        );
        let find_maxdepth_xargs_echo_pipe = format!(
            "find {} -maxdepth 1 -type f | xargs echo",
            path_string(&small_find_root)
        );
        let find_maxdepth_two_sort_tail_pipe = format!(
            "find {} -maxdepth 2 -type f | sort | tail -n 1",
            path_string(&small_find_root)
        );
        let find_maxdepth_two_name_grep_wc_pipe = format!(
            "find {} -maxdepth 2 -type f -name '*.rs' | grep nested-small | wc -l",
            path_string(&small_find_root)
        );
        let find_xargs_pipe = format!(
            "find {} -type f -name '*.rs' | xargs wc -l",
            path_string(&find_root)
        );
        let find_xargs_echo_pipe = format!(
            "find {} -type f -name '*.rs' | xargs echo",
            path_string(&find_root)
        );
        let find_xargs_default_pipe = format!(
            "find {} -type f -name '*.rs' | xargs",
            path_string(&find_root)
        );
        let find_grep_xargs_echo_pipe = format!(
            "find {} -type f -name '*.rs' | grep item-019 | xargs echo",
            path_string(&find_root)
        );
        let find_grep_xargs_pipe = format!(
            "find {} -type f -name '*.rs' | grep item-019 | xargs wc -l",
            path_string(&find_root)
        );
        let find_grep_wc_pipe = format!(
            "find {} -type f -name '*.rs' | grep item-019 | wc -l",
            path_string(&find_root)
        );
        let find_grep_head_pipe = format!(
            "find {} -type f -name '*.rs' | grep item-019 | head -n 50",
            path_string(&find_root)
        );
        let find_grep_sort_uniq_wc_pipe = format!(
            "find {} -type f -name '*.rs' | grep item-019 | sort | uniq | wc -l",
            path_string(&find_root)
        );
        let find_grep_sort_xargs_echo_pipe = format!(
            "find {} -type f -name '*.rs' | grep item-019 | sort | xargs echo",
            path_string(&find_root)
        );
        let find_grep_sort_xargs_pipe = format!(
            "find {} -type f -name '*.rs' | grep item-019 | sort | xargs wc -l",
            path_string(&find_root)
        );
        let find_grep_sort_xargs_sort_pipe = format!(
            "find {} -type f -name '*.rs' | grep item-019 | sort | xargs wc -l | sort",
            path_string(&find_root)
        );
        let find_wc_pipe = format!(
            "find {} -type f -name '*.rs' | wc -l",
            path_string(&find_root)
        );
        let find_head_pipe = format!(
            "find {} -type f -name '*.rs' | head -n 50",
            path_string(&find_root)
        );
        let find_tail_pipe = format!(
            "find {} -type f -name '*.rs' | tail -n 50",
            path_string(&find_root)
        );
        let find_sort_pipe = format!(
            "find {} -type f -name '*.rs' | sort",
            path_string(&find_root)
        );
        let find_sort_uniq_pipe = format!(
            "find {} -type f -name '*.rs' | sort | uniq",
            path_string(&find_root)
        );
        let find_sort_uniq_wc_pipe = format!(
            "find {} -type f -name '*.rs' | sort | uniq | wc -l",
            path_string(&find_root)
        );
        let find_sort_wc_pipe = format!(
            "find {} -type f -name '*.rs' | sort | wc -l",
            path_string(&find_root)
        );
        let find_sort_xargs_echo_pipe = format!(
            "find {} -type f -name '*.rs' | sort | xargs echo",
            path_string(&find_root)
        );
        let find_sort_xargs_pipe = format!(
            "find {} -type f -name '*.rs' | sort | xargs wc -l",
            path_string(&find_root)
        );
        let find_sort_xargs_sort_tail_pipe = format!(
            "find {} -type f -name '*.rs' | sort | xargs wc -l | sort | tail -n 1",
            path_string(&find_root)
        );
        let find_sort_head_pipe = format!(
            "find {} -type f -name '*.rs' | sort | head -n 50",
            path_string(&find_root)
        );
        let find_sort_tail_pipe = format!(
            "find {} -type f -name '*.rs' | sort | tail -n 50",
            path_string(&find_root)
        );
        let run_ls = format!("ls -1 {}", path_string(&list_dir));
        let run_cat = format!("cat {}", path_string(&cat_file));
        let run_uniq = format!("uniq {}", path_string(&byte_window_file));
        let run_find = format!("find {} -type f -name '*.txt'", path_string(&find_root));
        let run_du = format!("du -sk {}", path_string(&find_root));
        let run_sort = format!("sort {}", path_string(&sort_file));
        let run_cut = format!("cut -d, -f1 {}", path_string(&cut_file));
        let cut_stdin_wc_pipe = "cut -d, -f1 | wc -l".to_string();
        let run_tr = "tr a-z A-Z".to_string();
        let run_sed = format!("sed -n 2500,7500p {}", path_string(&sed_file));
        let cat_sed_pipe = format!("cat {} | sed -n 2500,7500p", path_string(&sed_file));
        let cat_sed_wc_pipe = format!("cat {} | sed -n 2500,7500p | wc -l", path_string(&sed_file));
        let cat_sed_head_pipe = format!(
            "cat {} | sed -n 2500,7500p | head -n 50",
            path_string(&sed_file)
        );
        let cat_sed_sort_uniq_wc_pipe = format!(
            "cat {} | sed -n 2500,7500p | sort | uniq | wc -l",
            path_string(&sed_file)
        );
        let cat_sed_xargs_wc_pipe = format!(
            "cat {} | sed -n 1,500p | xargs wc -l",
            path_string(&xargs_wc_input)
        );
        let cat_sed_grep_sort_xargs_echo_pipe = format!(
            "cat {} | sed -n 2500,7500p | grep NEEDLE | sort | xargs echo",
            path_string(&sed_file)
        );
        let run_grep = format!("grep -R NEEDLE {}", path_string(&grep_root));
        let run_grep_file = format!("grep NEEDLE {}", path_string(&sed_file));
        let mut wc_cap_args = strings(["wc", "-l"]);
        wc_cap_args.extend(wc_files.iter().cloned());
        let mut wc_original_args = strings(["-l"]);
        wc_original_args.extend(wc_files.iter().cloned());
        let mut wc_all_cap_args = strings(["wc"]);
        wc_all_cap_args.extend(wc_files.iter().cloned());
        let wc_all_original_args = wc_files.clone();
        let mut wc_bytes_cap_args = strings(["wc", "-c"]);
        wc_bytes_cap_args.extend(wc_files.iter().cloned());
        let mut wc_bytes_original_args = strings(["-c"]);
        wc_bytes_original_args.extend(wc_files.iter().cloned());
        let mut wc_words_cap_args = strings(["wc", "-w"]);
        wc_words_cap_args.extend(wc_files.iter().cloned());
        let mut wc_words_original_args = strings(["-w"]);
        wc_words_original_args.extend(wc_files.iter().cloned());
        let mut echo_cap_args = strings(["echo"]);
        echo_cap_args.extend(echo_words.iter().cloned());
        let echo_original_args = echo_words.clone();
        let mut printf_cap_args = strings(["printf", "%s\\n"]);
        printf_cap_args.extend(printf_words.iter().cloned());
        let mut printf_original_args = strings(["%s\\n"]);
        printf_original_args.extend(printf_words.iter().cloned());

        vec![
            Scenario {
                id: "true_noop",
                command: "true",
                description: "zero-argument success exit",
                gate: Gate::Takeover,
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
                gate: Gate::Takeover,
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
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["pwd"]),
                original_program: "/bin/pwd".to_string(),
                original_args: vec![],
                stdin_file: None,
            },
            Scenario {
                id: "echo_many_words",
                command: "echo",
                description: "2,000 plain words",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: echo_cap_args,
                original_program: "/bin/echo".to_string(),
                original_args: echo_original_args,
                stdin_file: None,
            },
            Scenario {
                id: "printf_many_lines",
                command: "printf",
                description: "2,000 %s newline arguments",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: printf_cap_args,
                original_program: "/usr/bin/printf".to_string(),
                original_args: printf_original_args,
                stdin_file: None,
            },
            Scenario {
                id: "seq_integer_range",
                command: "seq",
                description: "integer range 1 to 200,000",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["seq", "1", "200000"]),
                original_program: "/usr/bin/seq".to_string(),
                original_args: strings(["1", "200000"]),
                stdin_file: None,
            },
            Scenario {
                id: "whoami_effective_user",
                command: "whoami",
                description: "effective user name",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["whoami"]),
                original_program: "/usr/bin/whoami".to_string(),
                original_args: vec![],
                stdin_file: None,
            },
            Scenario {
                id: "id_default",
                command: "id",
                description: "default identity summary",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["id"]),
                original_program: "/usr/bin/id".to_string(),
                original_args: vec![],
                stdin_file: None,
            },
            Scenario {
                id: "id_effective_uid",
                command: "id",
                description: "effective user id",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["id", "-u"]),
                original_program: "/usr/bin/id".to_string(),
                original_args: strings(["-u"]),
                stdin_file: None,
            },
            Scenario {
                id: "id_effective_user_name",
                command: "id",
                description: "effective user name",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["id", "-un"]),
                original_program: "/usr/bin/id".to_string(),
                original_args: strings(["-un"]),
                stdin_file: None,
            },
            Scenario {
                id: "id_effective_gid",
                command: "id",
                description: "effective group id",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["id", "-g"]),
                original_program: "/usr/bin/id".to_string(),
                original_args: strings(["-g"]),
                stdin_file: None,
            },
            Scenario {
                id: "id_effective_group_name",
                command: "id",
                description: "effective group name",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["id", "-gn"]),
                original_program: "/usr/bin/id".to_string(),
                original_args: strings(["-gn"]),
                stdin_file: None,
            },
            Scenario {
                id: "id_group_ids",
                command: "id",
                description: "supplementary group id list",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["id", "-G"]),
                original_program: "/usr/bin/id".to_string(),
                original_args: strings(["-G"]),
                stdin_file: None,
            },
            Scenario {
                id: "id_group_names",
                command: "id",
                description: "supplementary group name list",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["id", "-Gn"]),
                original_program: "/usr/bin/id".to_string(),
                original_args: strings(["-Gn"]),
                stdin_file: None,
            },
            Scenario {
                id: "uname_machine",
                command: "uname",
                description: "machine architecture field",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["uname", "-m"]),
                original_program: "/usr/bin/uname".to_string(),
                original_args: strings(["-m"]),
                stdin_file: None,
            },
            Scenario {
                id: "uname_processor",
                command: "uname",
                description: "processor architecture field",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["uname", "-p"]),
                original_program: "/usr/bin/uname".to_string(),
                original_args: strings(["-p"]),
                stdin_file: None,
            },
            Scenario {
                id: "uname_all",
                command: "uname",
                description: "all utsname fields",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["uname", "-a"]),
                original_program: "/usr/bin/uname".to_string(),
                original_args: strings(["-a"]),
                stdin_file: None,
            },
            Scenario {
                id: "hostname_name",
                command: "hostname",
                description: "kernel hostname",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["hostname"]),
                original_program: "/bin/hostname".to_string(),
                original_args: Vec::new(),
                stdin_file: None,
            },
            Scenario {
                id: "test_file_regular",
                command: "test",
                description: "test -f regular file",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["test", "-f", &path_string(&cat_file)]),
                original_program: "/bin/test".to_string(),
                original_args: strings(["-f", &path_string(&cat_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "test_int_compare",
                command: "test",
                description: "integer comparison predicate",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["test", "500", "-gt", "20"]),
                original_program: "/bin/test".to_string(),
                original_args: strings(["500", "-gt", "20"]),
                stdin_file: None,
            },
            Scenario {
                id: "bracket_directory",
                command: "[",
                description: "[ -d directory ] predicate",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["[", "-d", &path_string(&find_root), "]"]),
                original_program: "/bin/[".to_string(),
                original_args: strings(["-d", &path_string(&find_root), "]"]),
                stdin_file: None,
            },
            Scenario {
                id: "which_external_and_builtin",
                command: "which",
                description: "which path lookup over external and shell builtin names",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["which", "sh", "echo"]),
                original_program: "/usr/bin/which".to_string(),
                original_args: strings(["sh", "echo"]),
                stdin_file: None,
            },
            Scenario {
                id: "which_all_external_and_builtin",
                command: "which",
                description: "which -a path lookup over external and shell builtin names",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["which", "-a", "sh", "echo"]),
                original_program: "/usr/bin/which".to_string(),
                original_args: strings(["-a", "sh", "echo"]),
                stdin_file: None,
            },
            Scenario {
                id: "command_v_external_and_builtin",
                command: "command",
                description: "command -v lookup over external and shell builtin names",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["command", "-v", "sh", "echo"]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", "command -v sh echo"]),
                stdin_file: None,
            },
            Scenario {
                id: "env_all",
                command: "env",
                description: "environment listing",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["env"]),
                original_program: "/usr/bin/env".to_string(),
                original_args: Vec::new(),
                stdin_file: None,
            },
            Scenario {
                id: "printenv_all",
                command: "printenv",
                description: "print all environment values",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["printenv"]),
                original_program: "/usr/bin/printenv".to_string(),
                original_args: Vec::new(),
                stdin_file: None,
            },
            Scenario {
                id: "printenv_path",
                command: "printenv",
                description: "print one environment value",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["printenv", "PATH"]),
                original_program: "/usr/bin/printenv".to_string(),
                original_args: strings(["PATH"]),
                stdin_file: None,
            },
            Scenario {
                id: "basename_path",
                command: "basename",
                description: "long path basename with suffix",
                gate: Gate::Takeover,
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
                gate: Gate::Takeover,
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
                id: "ls_small_takeover",
                command: "ls",
                description: "small ls takeover path",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["ls", "-1", &path_string(&small_list_dir)]),
                original_program: "/bin/ls".to_string(),
                original_args: strings(["-1", &path_string(&small_list_dir)]),
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
                gate: Gate::Takeover,
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
                gate: Gate::Takeover,
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
                id: "wc_lines_many_files",
                command: "wc",
                description: "2,000 regular files, wc -l aggregate",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: wc_cap_args,
                original_program: "/usr/bin/wc".to_string(),
                original_args: wc_original_args,
                stdin_file: None,
            },
            Scenario {
                id: "wc_all_many_files",
                command: "wc",
                description: "2,000 regular files, default wc aggregate",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: wc_all_cap_args,
                original_program: "/usr/bin/wc".to_string(),
                original_args: wc_all_original_args,
                stdin_file: None,
            },
            Scenario {
                id: "wc_bytes_many_files",
                command: "wc",
                description: "2,000 regular files, wc -c aggregate",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: wc_bytes_cap_args,
                original_program: "/usr/bin/wc".to_string(),
                original_args: wc_bytes_original_args,
                stdin_file: None,
            },
            Scenario {
                id: "wc_words_many_files",
                command: "wc",
                description: "2,000 regular files, wc -w aggregate",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: wc_words_cap_args,
                original_program: "/usr/bin/wc".to_string(),
                original_args: wc_words_original_args,
                stdin_file: None,
            },
            Scenario {
                id: "wc_stdin_bytes",
                command: "wc",
                description: "stdin byte count over 8.5 MiB input",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["wc", "-c"]),
                original_program: "/usr/bin/wc".to_string(),
                original_args: strings(["-c"]),
                stdin_file: Some(cat_file.clone()),
            },
            Scenario {
                id: "head_byte_window",
                command: "head",
                description: "first 64 MiB byte window",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["head", "-c", "67108864", &path_string(&byte_window_file)]),
                original_program: "/usr/bin/head".to_string(),
                original_args: strings(["-c", "67108864", &path_string(&byte_window_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "head_stdin_lines",
                command: "head",
                description: "stdin first 50 lines over 8.5 MiB input",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["head", "-n", "50"]),
                original_program: "/usr/bin/head".to_string(),
                original_args: strings(["-n", "50"]),
                stdin_file: Some(cat_file.clone()),
            },
            Scenario {
                id: "tail_byte_window",
                command: "tail",
                description: "last 64 MiB byte window",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["tail", "-c", "67108864", &path_string(&byte_window_file)]),
                original_program: "/usr/bin/tail".to_string(),
                original_args: strings(["-c", "67108864", &path_string(&byte_window_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "tail_stdin_lines",
                command: "tail",
                description: "stdin last 50 lines over 8.5 MiB input",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["tail", "-n", "50"]),
                original_program: "/usr/bin/tail".to_string(),
                original_args: strings(["-n", "50"]),
                stdin_file: Some(sed_file.clone()),
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
                id: "uniq_stdin_long_line",
                command: "uniq",
                description: "stdin adjacent duplicate filtering over 64 MiB single-line input",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["uniq"]),
                original_program: "/usr/bin/uniq".to_string(),
                original_args: Vec::new(),
                stdin_file: Some(byte_window_file.clone()),
            },
            Scenario {
                id: "sort_stdin_lines",
                command: "sort",
                description: "stdin sort over 500,000 reverse-ordered lines",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["sort"]),
                original_program: "/usr/bin/sort".to_string(),
                original_args: Vec::new(),
                stdin_file: Some(sort_file.clone()),
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
                id: "find_name_type_small_takeover",
                command: "find",
                description: "small find takeover path",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings([
                    "find",
                    &path_string(&small_find_root),
                    "-type",
                    "f",
                    "-name",
                    "*.txt",
                ]),
                original_program: "/usr/bin/find".to_string(),
                original_args: strings([
                    &path_string(&small_find_root),
                    "-type",
                    "f",
                    "-name",
                    "*.txt",
                ]),
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
                id: "sort_small_takeover",
                command: "sort",
                description: "small sort takeover path",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["sort", &path_string(&small_sort_file)]),
                original_program: "/usr/bin/sort".to_string(),
                original_args: strings([&path_string(&small_sort_file)]),
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
                id: "cut_field_csv",
                command: "cut",
                description: "first CSV field from 200,000-line file",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["cut", "-d,", "-f1", &path_string(&cut_file)]),
                original_program: "/usr/bin/cut".to_string(),
                original_args: strings(["-d,", "-f1", &path_string(&cut_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "cut_stdin_field_csv",
                command: "cut",
                description: "first CSV field from 200,000-line stdin stream",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["cut", "-d,", "-f1"]),
                original_program: "/usr/bin/cut".to_string(),
                original_args: strings(["-d,", "-f1"]),
                stdin_file: Some(cut_file.clone()),
            },
            Scenario {
                id: "run_string_cut_field_csv",
                command: "run",
                description: "hook string: cut first CSV field from 200,000-line file",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_cut]),
                original_program: "/usr/bin/cut".to_string(),
                original_args: strings(["-d,", "-f1", &path_string(&cut_file)]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cut_stdin_wc_lines",
                command: "pipe",
                description: "stdin cut first CSV field piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cut_stdin_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cut_stdin_wc_pipe]),
                stdin_file: Some(cut_file.clone()),
            },
            Scenario {
                id: "tr_uppercase",
                command: "tr",
                description: "uppercase 8.6 MiB stdin stream",
                gate: Gate::CpuWin,
                expected_exit_code: 0,
                cap_args: strings(["tr", "a-z", "A-Z"]),
                original_program: "/usr/bin/tr".to_string(),
                original_args: strings(["a-z", "A-Z"]),
                stdin_file: Some(tr_input.clone()),
            },
            Scenario {
                id: "tr_class_uppercase",
                command: "tr",
                description: "class uppercase 8.6 MiB stdin stream",
                gate: Gate::CpuWin,
                expected_exit_code: 0,
                cap_args: strings(["tr", "[:lower:]", "[:upper:]"]),
                original_program: "/usr/bin/tr".to_string(),
                original_args: strings(["[:lower:]", "[:upper:]"]),
                stdin_file: Some(tr_input.clone()),
            },
            Scenario {
                id: "tr_class_delete_digits",
                command: "tr",
                description: "delete digit class from 8.6 MiB stdin stream",
                gate: Gate::CpuWin,
                expected_exit_code: 0,
                cap_args: strings(["tr", "-d", "[:digit:]"]),
                original_program: "/usr/bin/tr".to_string(),
                original_args: strings(["-d", "[:digit:]"]),
                stdin_file: Some(tr_input.clone()),
            },
            Scenario {
                id: "run_string_tr_uppercase",
                command: "run",
                description: "hook string: tr uppercase 8.6 MiB stdin stream",
                gate: Gate::CpuWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_tr]),
                original_program: "/usr/bin/tr".to_string(),
                original_args: strings(["a-z", "A-Z"]),
                stdin_file: Some(tr_input.clone()),
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
                id: "sed_small_takeover",
                command: "sed",
                description: "small sed -n takeover path",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["sed", "-n", "1,2p", &path_string(&small_sed_file)]),
                original_program: "/usr/bin/sed".to_string(),
                original_args: strings(["-n", "1,2p", &path_string(&small_sed_file)]),
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
                id: "pipe_cat_sed",
                command: "pipe",
                description: "cat output piped to sed range print",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sed_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sed_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sed_wc",
                command: "pipe",
                description: "cat output piped to sed range print then wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sed_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sed_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sed_head",
                command: "pipe",
                description: "cat output piped to sed range print then head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sed_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sed_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sed_sort_uniq_wc",
                command: "pipe",
                description: "cat output piped to sed range print then sort, uniq, and wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sed_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sed_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sed_xargs_wc",
                command: "pipe",
                description: "cat path-list output piped to sed range print then xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sed_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sed_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sed_grep_sort_xargs_echo",
                command: "pipe",
                description:
                    "cat output piped to sed range print, literal grep, sort, and xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sed_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sed_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "awk_count_matches",
                command: "awk",
                description: "count NEEDLE matches in 120,000-line file",
                gate: Gate::DualWin,
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
                id: "awk_stdin_count_matches",
                command: "awk",
                description: "count NEEDLE matches from stdin over 120,000 lines",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["awk", "/NEEDLE/ { c++ } END { print c }"]),
                original_program: "/usr/bin/awk".to_string(),
                original_args: strings(["/NEEDLE/ { c++ } END { print c }"]),
                stdin_file: Some(sed_file.clone()),
            },
            Scenario {
                id: "awk_stdin_first_field",
                command: "awk",
                description: "first-field extraction from stdin over 120,000 lines",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["awk", "{ print $1 }"]),
                original_program: "/usr/bin/awk".to_string(),
                original_args: strings(["{ print $1 }"]),
                stdin_file: Some(sed_file.clone()),
            },
            Scenario {
                id: "awk_stdin_second_field",
                command: "awk",
                description: "second-field extraction from stdin over 120,000 lines",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["awk", "{ print $2 }"]),
                original_program: "/usr/bin/awk".to_string(),
                original_args: strings(["{ print $2 }"]),
                stdin_file: Some(sed_file.clone()),
            },
            Scenario {
                id: "xargs_echo_words",
                command: "xargs",
                description: "xargs echo over 20,000 input words",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["xargs", "echo"]),
                original_program: "/usr/bin/xargs".to_string(),
                original_args: strings(["echo"]),
                stdin_file: Some(xargs_input.clone()),
            },
            Scenario {
                id: "xargs_default_echo_words",
                command: "xargs",
                description: "default xargs echo over 20,000 input words",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["xargs"]),
                original_program: "/usr/bin/xargs".to_string(),
                original_args: vec![],
                stdin_file: Some(xargs_input.clone()),
            },
            Scenario {
                id: "xargs_n1_echo_words",
                command: "xargs",
                description: "xargs -n 1 echo over 20,000 input words",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["xargs", "-n", "1", "echo"]),
                original_program: "/usr/bin/xargs".to_string(),
                original_args: strings(["-n", "1", "echo"]),
                stdin_file: Some(xargs_input.clone()),
            },
            Scenario {
                id: "xargs_n2_echo_words",
                command: "xargs",
                description: "xargs -n 2 echo over 20,000 input words",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["xargs", "-n", "2", "echo"]),
                original_program: "/usr/bin/xargs".to_string(),
                original_args: strings(["-n", "2", "echo"]),
                stdin_file: Some(xargs_input.clone()),
            },
            Scenario {
                id: "xargs_wc_lines",
                command: "xargs",
                description: "xargs wc -l over 2,000 input paths",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["xargs", "wc", "-l"]),
                original_program: "/usr/bin/xargs".to_string(),
                original_args: strings(["wc", "-l"]),
                stdin_file: Some(xargs_wc_input.clone()),
            },
            Scenario {
                id: "pipe_cat_wc",
                command: "pipe",
                description: "cat output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_wc_bytes",
                command: "pipe",
                description: "cat output piped to wc -c",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_wc_bytes_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_wc_bytes_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_echo_wc",
                command: "pipe",
                description: "echo output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &echo_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &echo_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_echo_wc_words",
                command: "pipe",
                description: "echo output piped to wc -w",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &echo_wc_words_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &echo_wc_words_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_echo_head",
                command: "pipe",
                description: "echo -n output piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &echo_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &echo_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_echo_tail",
                command: "pipe",
                description: "echo -n output piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &echo_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &echo_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_echo_tr",
                command: "pipe",
                description: "echo output piped to tr uppercase",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &echo_tr_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &echo_tr_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_echo_awk_xargs",
                command: "pipe",
                description: "echo output piped through awk first-field to default xargs",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &echo_awk_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &echo_awk_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_echo_xargs_echo",
                command: "pipe",
                description: "echo output piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &echo_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &echo_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_echo_xargs_wc",
                command: "pipe",
                description: "echo path output piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &echo_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &echo_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_xargs_stdin_wc",
                command: "pipe",
                description: "xargs echo stdin output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &xargs_stdin_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &xargs_stdin_wc_pipe]),
                stdin_file: Some(sort_file.clone()),
            },
            Scenario {
                id: "pipe_xargs_n1_stdin_wc",
                command: "pipe",
                description: "xargs -n 1 echo stdin output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &xargs_n1_stdin_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &xargs_n1_stdin_wc_pipe]),
                stdin_file: Some(sort_file.clone()),
            },
            Scenario {
                id: "pipe_xargs_stdin_grep_wc",
                command: "pipe",
                description: "xargs echo stdin output piped through grep to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &xargs_stdin_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &xargs_stdin_grep_wc_pipe]),
                stdin_file: Some(xargs_input.clone()),
            },
            Scenario {
                id: "pipe_grep_stdin_wc",
                command: "pipe",
                description: "grep stdin output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_stdin_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_stdin_wc_pipe]),
                stdin_file: Some(sed_file.clone()),
            },
            Scenario {
                id: "pipe_printf_wc",
                command: "pipe",
                description: "printf generated lines piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_wc_bytes",
                command: "pipe",
                description: "printf generated lines piped to wc -c",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_wc_bytes_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_wc_bytes_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_head",
                command: "pipe",
                description: "printf generated lines piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_tail",
                command: "pipe",
                description: "printf generated lines piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_awk_wc",
                command: "pipe",
                description: "printf generated lines piped through awk first-field to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_awk_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_awk_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_awk_sort_uniq",
                command: "pipe",
                description: "printf generated lines piped through awk first-field to sort uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_awk_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_awk_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep",
                command: "pipe",
                description: "printf generated lines piped to grep",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_wc",
                command: "pipe",
                description: "printf generated lines piped through grep to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_head",
                command: "pipe",
                description: "printf generated lines piped through grep to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_tail",
                command: "pipe",
                description: "printf generated lines piped through grep to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort",
                command: "pipe",
                description: "printf generated lines piped through grep to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort_uniq",
                command: "pipe",
                description: "printf generated lines piped through grep, sort, and uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort_uniq_wc",
                command: "pipe",
                description: "printf generated lines piped through grep, sort, uniq, and wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort_uniq_wc_words",
                command: "pipe",
                description: "printf generated lines piped through grep, sort, uniq, and wc -w",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_uniq_wc_words_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_uniq_wc_words_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort_uniq_head",
                command: "pipe",
                description: "printf generated lines piped through grep, sort, uniq, and head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_uniq_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_uniq_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort_uniq_xargs_wc",
                command: "pipe",
                description:
                    "printf generated paths piped through grep, sort, uniq, and xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_uniq_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_uniq_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort_wc",
                command: "pipe",
                description: "printf generated lines piped through grep and sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort_head",
                command: "pipe",
                description: "printf generated lines piped through grep and sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort_tail",
                command: "pipe",
                description: "printf generated lines piped through grep and sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_sort_xargs_echo",
                command: "pipe",
                description: "printf generated lines piped through grep and sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_grep_xargs_echo",
                command: "pipe",
                description: "printf generated lines piped through grep to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_grep_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_grep_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_tr",
                command: "pipe",
                description: "printf generated lines piped to tr uppercase",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_tr_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_tr_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort",
                command: "pipe",
                description: "printf generated lines piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort_uniq",
                command: "pipe",
                description: "printf generated lines piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort_uniq_wc",
                command: "pipe",
                description: "printf generated lines piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort_uniq_head",
                command: "pipe",
                description: "printf generated lines piped through sort and uniq to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_uniq_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_uniq_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort_uniq_xargs_wc",
                command: "pipe",
                description: "printf generated paths piped through sort, uniq, and xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_uniq_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_uniq_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort_wc",
                command: "pipe",
                description: "printf generated lines piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort_head",
                command: "pipe",
                description: "printf generated lines piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort_tail",
                command: "pipe",
                description: "printf generated lines piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort_xargs_echo",
                command: "pipe",
                description: "printf generated lines piped through sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_sort_xargs_wc",
                command: "pipe",
                description: "printf generated paths piped through sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_xargs_echo",
                command: "pipe",
                description: "printf generated lines piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_xargs_wc",
                command: "pipe",
                description: "printf generated paths piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_wc",
                command: "pipe",
                description: "seq generated lines piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_head",
                command: "pipe",
                description: "seq generated lines piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_tail",
                command: "pipe",
                description: "seq generated lines piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_sort",
                command: "pipe",
                description: "seq generated lines piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_sort_uniq",
                command: "pipe",
                description: "seq generated lines piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_sort_uniq_wc",
                command: "pipe",
                description: "seq generated lines piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_sort_uniq_head",
                command: "pipe",
                description: "seq generated lines piped through sort and uniq to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_sort_uniq_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_sort_uniq_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_sort_uniq_sort_xargs_echo",
                command: "pipe",
                description: "seq generated lines piped through sort, uniq, sort, and xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_sort_uniq_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_sort_uniq_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_sort_wc",
                command: "pipe",
                description: "seq generated lines piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_sort_head",
                command: "pipe",
                description: "seq generated lines piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_sort_tail",
                command: "pipe",
                description: "seq generated lines piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_sort_xargs_echo",
                command: "pipe",
                description: "seq generated lines piped through sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep",
                command: "pipe",
                description: "seq generated lines piped to literal grep",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_wc",
                command: "pipe",
                description: "seq generated lines piped through literal grep to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_head",
                command: "pipe",
                description: "seq generated lines piped through literal grep to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_tail",
                command: "pipe",
                description: "seq generated lines piped through literal grep to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_sort",
                command: "pipe",
                description: "seq generated lines piped through literal grep to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_sort_uniq",
                command: "pipe",
                description: "seq generated lines piped through literal grep, sort, and uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_sort_uniq_wc",
                command: "pipe",
                description:
                    "seq generated lines piped through literal grep, sort, uniq, and wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_sort_uniq_head",
                command: "pipe",
                description:
                    "seq generated lines piped through literal grep, sort, uniq, and head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_sort_uniq_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_sort_uniq_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_sort_uniq_sort_xargs_echo",
                command: "pipe",
                description:
                    "seq generated lines piped through literal grep, sort, uniq, sort, and xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_sort_uniq_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_sort_uniq_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_sort_wc",
                command: "pipe",
                description: "seq generated lines piped through literal grep and sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_sort_head",
                command: "pipe",
                description: "seq generated lines piped through literal grep and sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_sort_tail",
                command: "pipe",
                description: "seq generated lines piped through literal grep and sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_sort_xargs_echo",
                command: "pipe",
                description:
                    "seq generated lines piped through literal grep and sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_grep_xargs_echo",
                command: "pipe",
                description: "seq generated lines piped through literal grep to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_grep_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_grep_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_seq_xargs_echo",
                command: "pipe",
                description: "seq generated lines piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &seq_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &seq_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_yes_head",
                command: "pipe",
                description: "yes generated lines piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &yes_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &yes_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_wc",
                command: "pipe",
                description: "which output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_head",
                command: "pipe",
                description: "which output piped to head",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_tail",
                command: "pipe",
                description: "which output piped to tail",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_grep_wc",
                command: "pipe",
                description: "which output piped through literal grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_xargs_echo",
                command: "pipe",
                description: "which output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_sort_wc",
                command: "pipe",
                description: "which output piped through sort to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_sort_xargs_echo",
                command: "pipe",
                description: "which output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_all_wc",
                command: "pipe",
                description: "which -a output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_all_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_all_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_all_xargs_echo",
                command: "pipe",
                description: "which -a output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_all_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_all_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_which_all_sort_xargs_echo",
                command: "pipe",
                description: "which -a output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &which_all_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &which_all_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_command_v_wc",
                command: "pipe",
                description: "command -v output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &command_v_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &command_v_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_command_v_head",
                command: "pipe",
                description: "command -v output piped to head",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &command_v_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &command_v_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_command_v_tail",
                command: "pipe",
                description: "command -v output piped to tail",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &command_v_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &command_v_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_command_v_grep_wc",
                command: "pipe",
                description: "command -v output piped through literal grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &command_v_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &command_v_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_command_v_xargs_echo",
                command: "pipe",
                description: "command -v output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &command_v_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &command_v_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_command_v_sort_wc",
                command: "pipe",
                description: "command -v output piped through sort to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &command_v_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &command_v_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_command_v_sort_xargs_echo",
                command: "pipe",
                description: "command -v output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &command_v_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &command_v_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printenv_path_wc",
                command: "pipe",
                description: "printenv PATH piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printenv_path_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printenv_path_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printenv_path_grep",
                command: "pipe",
                description: "printenv PATH piped to literal grep",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printenv_path_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printenv_path_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printenv_path_grep_wc",
                command: "pipe",
                description: "printenv PATH piped through literal grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printenv_path_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printenv_path_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printenv_path_xargs_echo",
                command: "pipe",
                description: "printenv PATH piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printenv_path_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printenv_path_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printenv_path_sort_xargs_echo",
                command: "pipe",
                description: "printenv PATH piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printenv_path_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printenv_path_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_true_wc",
                command: "pipe",
                description: "true empty output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &true_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &true_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_false_wc",
                command: "pipe",
                description: "false empty output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &false_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &false_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_false_grep_wc",
                command: "pipe",
                description: "false empty output piped through literal grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &false_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &false_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_true_xargs_echo",
                command: "pipe",
                description: "true empty output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &true_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &true_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_mkdir_wc",
                command: "pipe",
                description: "`mkdir -p ...` empty output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &mkdir_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &mkdir_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_mkdir_xargs_echo",
                command: "pipe",
                description: "`mkdir -p ...` empty output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &mkdir_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &mkdir_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_touch_wc",
                command: "pipe",
                description: "`touch ...` empty output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &touch_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &touch_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_touch_sort_xargs_echo",
                command: "pipe",
                description: "`touch ...` empty output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &touch_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &touch_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_test_wc",
                command: "pipe",
                description: "`test -f ...` empty output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &test_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &test_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_test_xargs_echo",
                command: "pipe",
                description: "`test ! -e ...` empty output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &test_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &test_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_bracket_sort_xargs_echo",
                command: "pipe",
                description: "`[ -d ... ]` empty output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &bracket_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &bracket_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_test_grep_wc",
                command: "pipe",
                description: "`test -d ...` empty output piped through grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &test_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &test_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_wc_xargs_echo",
                command: "pipe",
                description: "`wc -l ...` output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &wc_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &wc_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_wc_multi_wc",
                command: "pipe",
                description: "`wc -c ... ...` output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &wc_multi_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &wc_multi_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_wc_grep_wc",
                command: "pipe",
                description: "`wc -l ... ...` output piped through grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &wc_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &wc_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_wc_sort_xargs_echo",
                command: "pipe",
                description: "`wc -w ...` output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &wc_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &wc_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_wc_stdin_wc",
                command: "pipe",
                description: "`wc -l` stdin output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &wc_stdin_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &wc_stdin_wc_pipe]),
                stdin_file: Some(sort_file.clone()),
            },
            Scenario {
                id: "pipe_wc_stdin_grep_wc",
                command: "pipe",
                description: "`wc -w` stdin output piped through grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &wc_stdin_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &wc_stdin_grep_wc_pipe]),
                stdin_file: Some(sort_file.clone()),
            },
            Scenario {
                id: "pipe_wc_stdin_sort_xargs_echo",
                command: "pipe",
                description: "`wc -l` stdin output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &wc_stdin_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &wc_stdin_sort_xargs_echo_pipe]),
                stdin_file: Some(sort_file.clone()),
            },
            Scenario {
                id: "pipe_printf_literal_wc",
                command: "pipe",
                description: "`printf literal` output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_literal_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_literal_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_literal_grep_wc",
                command: "pipe",
                description: "`printf literal` output piped through grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_literal_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_literal_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_printf_literal_sort_xargs_echo",
                command: "pipe",
                description: "`printf literal` output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &printf_literal_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &printf_literal_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_du_wc",
                command: "pipe",
                description: "`du -sk ...` output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &du_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &du_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_du_xargs_echo",
                command: "pipe",
                description: "`du -sk ...` output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &du_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &du_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_du_grep_wc",
                command: "pipe",
                description: "`du -sk ...` output piped through grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &du_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &du_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_hostname_wc",
                command: "pipe",
                description: "hostname output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &hostname_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &hostname_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_hostname_head",
                command: "pipe",
                description: "hostname output piped to head",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &hostname_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &hostname_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_hostname_tail",
                command: "pipe",
                description: "hostname output piped to tail",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &hostname_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &hostname_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_hostname_grep_wc",
                command: "pipe",
                description: "hostname output piped through literal grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &hostname_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &hostname_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_hostname_sort",
                command: "pipe",
                description: "hostname output piped to sort",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &hostname_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &hostname_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_hostname_xargs_echo",
                command: "pipe",
                description: "hostname output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &hostname_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &hostname_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_hostname_sort_xargs_echo",
                command: "pipe",
                description: "hostname output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &hostname_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &hostname_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_wc",
                command: "pipe",
                description: "ls output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_head",
                command: "pipe",
                description: "ls output piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_tail",
                command: "pipe",
                description: "ls output piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_sort",
                command: "pipe",
                description: "ls output piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_sort_uniq",
                command: "pipe",
                description: "ls output piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_sort_uniq_wc",
                command: "pipe",
                description: "ls output piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_sort_wc",
                command: "pipe",
                description: "ls output piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_sort_head",
                command: "pipe",
                description: "ls output piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_sort_tail",
                command: "pipe",
                description: "ls output piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_grep",
                command: "pipe",
                description: "ls output piped to literal grep",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_grep_wc",
                command: "pipe",
                description: "ls output piped to literal grep then wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_grep_head",
                command: "pipe",
                description: "ls output piped to literal grep then head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_grep_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_grep_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_grep_tail",
                command: "pipe",
                description: "ls output piped to literal grep then tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_grep_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_grep_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_grep_sort",
                command: "pipe",
                description: "ls output piped to literal grep then sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_grep_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_grep_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_grep_sort_uniq_wc",
                command: "pipe",
                description: "ls output piped to literal grep, sort, uniq, then wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_grep_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_grep_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_grep_xargs_echo",
                command: "pipe",
                description: "ls output piped to literal grep then xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_grep_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_grep_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_grep_sort_xargs_echo",
                command: "pipe",
                description: "ls output piped to literal grep then sort then xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_xargs_echo",
                command: "pipe",
                description: "ls output piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_sort_xargs_echo",
                command: "pipe",
                description: "ls output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_all_wc",
                command: "pipe",
                description: "ls -a output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_all_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_all_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_all_grep_wc",
                command: "pipe",
                description: "ls -a output piped to literal grep then wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_all_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_all_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_all_sort_tail",
                command: "pipe",
                description: "ls -a output piped through sort to tail",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_all_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_all_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_all_xargs_echo",
                command: "pipe",
                description: "ls -a output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_all_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_all_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_all_sort_xargs_echo",
                command: "pipe",
                description: "ls -a output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_all_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_all_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_almost_wc",
                command: "pipe",
                description: "ls -A output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_almost_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_almost_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_almost_grep_wc",
                command: "pipe",
                description: "ls -A output piped to literal grep then wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_almost_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_almost_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_almost_sort_tail",
                command: "pipe",
                description: "ls -A output piped through sort to tail",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_almost_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_almost_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_almost_xargs_echo",
                command: "pipe",
                description: "ls -A output piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_almost_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_almost_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_ls_almost_sort_xargs_echo",
                command: "pipe",
                description: "ls -A output piped through sort to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &ls_almost_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &ls_almost_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_uniq",
                command: "pipe",
                description: "sort output piped to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_uniq_wc",
                command: "pipe",
                description: "sort output piped through uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_uniq_wc_bytes",
                command: "pipe",
                description: "sort output piped through uniq to wc -c",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_uniq_wc_bytes_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_uniq_wc_bytes_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_grep",
                command: "pipe",
                description: "sort output piped through literal grep",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_grep_wc",
                command: "pipe",
                description: "sort output piped through literal grep to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_grep_xargs_wc",
                command: "pipe",
                description: "sort path-list output piped through literal grep to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_grep_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_grep_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_head",
                command: "pipe",
                description: "sort output piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_tail",
                command: "pipe",
                description: "sort output piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_wc",
                command: "pipe",
                description: "sort output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_wc_words",
                command: "pipe",
                description: "sort output piped to wc -w",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_wc_words_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_wc_words_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_wc",
                command: "pipe",
                description: "head file output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_stdin_wc",
                command: "pipe",
                description: "head stdin output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_stdin_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_stdin_wc_pipe]),
                stdin_file: Some(sort_file.clone()),
            },
            Scenario {
                id: "pipe_head_head",
                command: "pipe",
                description: "head file output piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_tail",
                command: "pipe",
                description: "head file output piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_sort",
                command: "pipe",
                description: "head file output piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_sort_uniq",
                command: "pipe",
                description: "head file output piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_sort_uniq_wc",
                command: "pipe",
                description: "head file output piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_sort_wc",
                command: "pipe",
                description: "head file output piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_sort_head",
                command: "pipe",
                description: "head file output piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_sort_tail",
                command: "pipe",
                description: "head file output piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_xargs_echo",
                command: "pipe",
                description: "head file output piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_xargs_wc",
                command: "pipe",
                description: "head path-list output piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_sort_xargs_echo",
                command: "pipe",
                description: "head file output piped through sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_sort_xargs_wc",
                command: "pipe",
                description: "head path-list output piped through sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep",
                command: "pipe",
                description: "head file output piped to literal grep",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_wc",
                command: "pipe",
                description: "head file output piped through literal grep to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_head",
                command: "pipe",
                description: "head file output piped through literal grep to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_tail",
                command: "pipe",
                description: "head file output piped through literal grep to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_sort",
                command: "pipe",
                description: "head file output piped through literal grep to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_sort_uniq",
                command: "pipe",
                description: "head file output piped through literal grep and sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_sort_uniq_wc",
                command: "pipe",
                description: "head file output piped through literal grep, sort, and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_sort_wc",
                command: "pipe",
                description: "head file output piped through literal grep and sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_sort_head",
                command: "pipe",
                description: "head file output piped through literal grep and sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_sort_tail",
                command: "pipe",
                description: "head file output piped through literal grep and sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_xargs_echo",
                command: "pipe",
                description: "head file output piped through literal grep to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_xargs_wc",
                command: "pipe",
                description: "head path-list output piped through literal grep to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_sort_xargs_echo",
                command: "pipe",
                description: "head file output piped through literal grep and sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_head_grep_sort_xargs_wc",
                command: "pipe",
                description:
                    "head path-list output piped through literal grep and sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &head_grep_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &head_grep_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_wc",
                command: "pipe",
                description: "tail file output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_stdin_wc",
                command: "pipe",
                description: "tail stdin output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_stdin_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_stdin_wc_pipe]),
                stdin_file: Some(sort_file.clone()),
            },
            Scenario {
                id: "pipe_tail_head",
                command: "pipe",
                description: "tail file output piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_tail",
                command: "pipe",
                description: "tail file output piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_sort",
                command: "pipe",
                description: "tail file output piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_sort_uniq",
                command: "pipe",
                description: "tail file output piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_sort_uniq_wc",
                command: "pipe",
                description: "tail file output piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_sort_wc",
                command: "pipe",
                description: "tail file output piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_sort_head",
                command: "pipe",
                description: "tail file output piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_sort_tail",
                command: "pipe",
                description: "tail file output piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_xargs_echo",
                command: "pipe",
                description: "tail file output piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_xargs_wc",
                command: "pipe",
                description: "tail path-list output piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_sort_xargs_echo",
                command: "pipe",
                description: "tail file output piped through sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_sort_xargs_wc",
                command: "pipe",
                description: "tail path-list output piped through sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep",
                command: "pipe",
                description: "tail file output piped to literal grep",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_wc",
                command: "pipe",
                description: "tail file output piped through literal grep to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_head",
                command: "pipe",
                description: "tail file output piped through literal grep to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_tail",
                command: "pipe",
                description: "tail file output piped through literal grep to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_sort",
                command: "pipe",
                description: "tail file output piped through literal grep to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_sort_uniq",
                command: "pipe",
                description: "tail file output piped through literal grep and sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_sort_uniq_wc",
                command: "pipe",
                description: "tail file output piped through literal grep, sort, and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_sort_wc",
                command: "pipe",
                description: "tail file output piped through literal grep and sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_sort_head",
                command: "pipe",
                description: "tail file output piped through literal grep and sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_sort_tail",
                command: "pipe",
                description: "tail file output piped through literal grep and sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_xargs_echo",
                command: "pipe",
                description: "tail file output piped through literal grep to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_xargs_wc",
                command: "pipe",
                description: "tail path-list output piped through literal grep to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_sort_xargs_echo",
                command: "pipe",
                description: "tail file output piped through literal grep and sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_tail_grep_sort_xargs_wc",
                command: "pipe",
                description:
                    "tail path-list output piped through literal grep and sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &tail_grep_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &tail_grep_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_head",
                command: "pipe",
                description: "cat output piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_tail",
                command: "pipe",
                description: "cat output piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_head_wc",
                command: "pipe",
                description: "cat output piped through head to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_head_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_head_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_tail_wc",
                command: "pipe",
                description: "cat output piped through tail to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_tail_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_tail_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_head_sort_uniq_wc",
                command: "pipe",
                description: "cat output piped through head, sort, uniq, and wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_head_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_head_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_tail_sort_uniq_wc",
                command: "pipe",
                description: "cat output piped through tail, sort, uniq, and wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_tail_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_tail_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_head_grep_sort_xargs_echo",
                command: "pipe",
                description: "cat output piped through head, grep, sort, and xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_head_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_head_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_tail_grep_sort_xargs_echo",
                command: "pipe",
                description: "cat output piped through tail, grep, sort, and xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_tail_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_tail_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_head_xargs_wc",
                command: "pipe",
                description: "cat path-list output piped through head to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_head_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_head_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_tail_xargs_wc",
                command: "pipe",
                description: "cat path-list output piped through tail to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_tail_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_tail_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep",
                command: "pipe",
                description: "cat output piped to grep",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_wc",
                command: "pipe",
                description: "cat output piped through literal grep to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_head",
                command: "pipe",
                description: "cat output piped through literal grep to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_tail",
                command: "pipe",
                description: "cat output piped through literal grep to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort",
                command: "pipe",
                description: "cat output piped through literal grep to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_uniq",
                command: "pipe",
                description: "cat output piped through literal grep and sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_uniq_wc",
                command: "pipe",
                description: "cat output piped through literal grep, sort, and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_uniq_head",
                command: "pipe",
                description: "cat output piped through literal grep, sort, and uniq to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_uniq_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_uniq_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_uniq_tail",
                command: "pipe",
                description: "cat output piped through literal grep, sort, and uniq to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_uniq_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_uniq_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_uniq_sort_xargs_echo",
                command: "pipe",
                description:
                    "cat output piped through literal grep, sort, and uniq to sorted xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_uniq_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_uniq_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_uniq_xargs_wc",
                command: "pipe",
                description:
                    "cat path-list output piped through literal grep, sort, and uniq to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_uniq_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_uniq_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_uniq_sort_xargs_wc",
                command: "pipe",
                description:
                    "cat path-list output piped through literal grep, sort, and uniq to sorted xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_uniq_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_uniq_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_wc",
                command: "pipe",
                description: "cat output piped through literal grep and sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_head",
                command: "pipe",
                description: "cat output piped through literal grep and sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_tail",
                command: "pipe",
                description: "cat output piped through literal grep and sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_cut",
                command: "pipe",
                description: "cat output piped to cut first CSV field",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_cut_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_cut_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_tr",
                command: "pipe",
                description: "cat output piped to tr uppercase",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_tr_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_tr_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_uniq",
                command: "pipe",
                description: "cat output piped to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_uniq_wc",
                command: "pipe",
                description: "cat output piped through uniq to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_uniq_wc",
                command: "pipe",
                description: "uniq output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_uniq_grep",
                command: "pipe",
                description: "uniq output piped through literal grep",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &uniq_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &uniq_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_uniq_grep_wc",
                command: "pipe",
                description: "uniq output piped through literal grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &uniq_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &uniq_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_uniq_grep_xargs_wc",
                command: "pipe",
                description: "uniq path-list output piped through literal grep to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &uniq_grep_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &uniq_grep_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort",
                command: "pipe",
                description: "cat output piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_uniq",
                command: "pipe",
                description: "cat output piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_uniq_wc",
                command: "pipe",
                description: "cat output piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_grep",
                command: "pipe",
                description: "cat output piped through sort and literal grep",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_grep_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_grep_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_grep_wc",
                command: "pipe",
                description: "cat output piped through sort and literal grep to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_grep_xargs_wc",
                command: "pipe",
                description: "cat path-list output piped through sort and literal grep to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_grep_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_grep_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_wc",
                command: "pipe",
                description: "cat output piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_head",
                command: "pipe",
                description: "cat output piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_tail",
                command: "pipe",
                description: "cat output piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_xargs_echo",
                command: "pipe",
                description: "cat output piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_xargs_echo",
                command: "pipe",
                description: "sort output piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_xargs_echo",
                command: "pipe",
                description: "cat output piped through sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_xargs_wc",
                command: "pipe",
                description: "cat path-list output piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_sort_xargs_wc",
                command: "pipe",
                description: "sort path-list output piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_sort_xargs_wc",
                command: "pipe",
                description: "cat path-list output piped through sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_xargs_echo",
                command: "pipe",
                description: "cat output piped through literal grep to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_xargs_wc",
                command: "pipe",
                description: "cat path-list output piped through literal grep to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_xargs_echo",
                command: "pipe",
                description: "cat output piped through literal grep and sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_grep_sort_xargs_wc",
                command: "pipe",
                description:
                    "cat path-list output piped through literal grep and sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_grep_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_grep_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_head",
                command: "pipe",
                description: "grep -R piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_tail",
                command: "pipe",
                description: "grep -R piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_sort",
                command: "pipe",
                description: "grep -R output piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_sort_uniq",
                command: "pipe",
                description: "grep -R output piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_sort_uniq_wc",
                command: "pipe",
                description: "grep -R output piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_sort_uniq_head",
                command: "pipe",
                description: "grep -R output piped through sort and uniq to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_sort_uniq_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_sort_uniq_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_sort_uniq_tail",
                command: "pipe",
                description: "grep -R output piped through sort and uniq to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_sort_uniq_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_sort_uniq_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_sort_uniq_sort_xargs_echo",
                command: "pipe",
                description: "grep -R output piped through sort and uniq to sorted xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_sort_uniq_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_sort_uniq_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_sort_wc",
                command: "pipe",
                description: "grep -R output piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_sort_head",
                command: "pipe",
                description: "grep -R output piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_sort_tail",
                command: "pipe",
                description: "grep -R output piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_wc",
                command: "pipe",
                description: "grep -R output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_wc",
                command: "pipe",
                description: "single-file grep output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_head",
                command: "pipe",
                description: "single-file grep output piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_tail",
                command: "pipe",
                description: "single-file grep output piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort",
                command: "pipe",
                description: "single-file grep output piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_uniq",
                command: "pipe",
                description: "single-file grep output piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_uniq_wc",
                command: "pipe",
                description: "single-file grep output piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_uniq_head",
                command: "pipe",
                description: "single-file grep output piped through sort and uniq to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_uniq_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_uniq_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_uniq_tail",
                command: "pipe",
                description: "single-file grep output piped through sort and uniq to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_uniq_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_uniq_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_uniq_sort_xargs_echo",
                command: "pipe",
                description: "single-file grep output piped through sort and uniq to sorted xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_uniq_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_uniq_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_uniq_xargs_wc",
                command: "pipe",
                description:
                    "single-file grep path output piped through sort and uniq to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_uniq_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_uniq_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_uniq_sort_xargs_wc",
                command: "pipe",
                description:
                    "single-file grep path output piped through sort and uniq to sorted xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_uniq_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_uniq_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_wc",
                command: "pipe",
                description: "single-file grep output piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_head",
                command: "pipe",
                description: "single-file grep output piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_tail",
                command: "pipe",
                description: "single-file grep output piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_xargs_echo",
                command: "pipe",
                description: "single-file grep output piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_xargs_wc",
                command: "pipe",
                description: "single-file grep path output piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_xargs_echo",
                command: "pipe",
                description: "single-file grep output piped through sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_sort_xargs_wc",
                command: "pipe",
                description: "single-file grep path output piped through sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_cut",
                command: "pipe",
                description: "single-file grep output piped through cut",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_cut_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_cut_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_cut_wc",
                command: "pipe",
                description: "single-file grep output piped through cut to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_cut_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_cut_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_cut_sort_uniq_wc",
                command: "pipe",
                description: "single-file grep output piped through cut, sort, and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_cut_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_cut_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_cut_xargs_echo",
                command: "pipe",
                description: "single-file grep output piped through cut to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_cut_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_cut_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_cut_xargs_wc",
                command: "pipe",
                description: "single-file grep path output piped through cut to xargs wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_cut_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_cut_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_awk",
                command: "pipe",
                description: "single-file grep output piped through awk print-field",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_awk_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_awk_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_awk_wc",
                command: "pipe",
                description: "single-file grep output piped through awk print-field to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_awk_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_awk_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_awk_sort_uniq_wc",
                command: "pipe",
                description:
                    "single-file grep output piped through awk print-field, sort, and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_awk_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_awk_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_awk_xargs_echo",
                command: "pipe",
                description: "single-file grep output piped through awk print-field to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_awk_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_awk_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_grep_file_awk_xargs_wc",
                command: "pipe",
                description:
                    "single-file grep path output piped through awk print-field to xargs wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &grep_file_awk_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &grep_file_awk_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_first_wc",
                command: "pipe",
                description: "awk first-field output piped to wc -l without a predicate",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_first_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_first_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_stdin_wc",
                command: "pipe",
                description: "awk first-field stdin output piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_stdin_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_stdin_wc_pipe]),
                stdin_file: Some(sed_file.clone()),
            },
            Scenario {
                id: "pipe_awk_first_sort_uniq_wc",
                command: "pipe",
                description:
                    "awk first-field output piped through sort and uniq to wc -l without a predicate",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_first_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_first_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_first_xargs_echo",
                command: "pipe",
                description: "awk first-field output piped to xargs echo without a predicate",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_first_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_first_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_first_xargs_wc",
                command: "pipe",
                description: "awk first-field path output piped to xargs wc -l without a predicate",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_first_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_first_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_first_grep_wc",
                command: "pipe",
                description: "awk first-field output piped through literal grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_first_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_first_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_first_grep_sort_uniq_wc",
                command: "pipe",
                description:
                    "awk first-field output piped through literal grep, sort, uniq, and wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_first_grep_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_first_grep_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_first_grep_xargs_wc",
                command: "pipe",
                description:
                    "awk first-field path output piped through literal grep to xargs wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_first_grep_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_first_grep_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_wc",
                command: "pipe",
                description: "awk output piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_head",
                command: "pipe",
                description: "awk output piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_tail",
                command: "pipe",
                description: "awk output piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort",
                command: "pipe",
                description: "awk output piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_uniq",
                command: "pipe",
                description: "awk output piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_uniq_wc",
                command: "pipe",
                description: "awk output piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_uniq_head",
                command: "pipe",
                description: "awk output piped through sort and uniq to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_uniq_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_uniq_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_uniq_sort_xargs_echo",
                command: "pipe",
                description: "awk output piped through sort and uniq to sorted xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_uniq_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_uniq_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_uniq_xargs_wc",
                command: "pipe",
                description: "awk output piped through sort and uniq to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_uniq_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_uniq_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_uniq_sort_xargs_wc",
                command: "pipe",
                description: "awk output piped through sort and uniq to sorted xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_uniq_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_uniq_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_wc",
                command: "pipe",
                description: "awk output piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_head",
                command: "pipe",
                description: "awk output piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_tail",
                command: "pipe",
                description: "awk output piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_xargs",
                command: "pipe",
                description: "awk output piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_xargs_wc",
                command: "pipe",
                description: "awk path output piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_xargs",
                command: "pipe",
                description: "awk output piped through sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_awk_sort_xargs_wc",
                command: "pipe",
                description: "awk path output piped through sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &awk_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &awk_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_awk",
                command: "pipe",
                description: "cat output piped to awk print-field",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_awk_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_awk_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_awk_wc",
                command: "pipe",
                description: "cat output piped to awk and wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_awk_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_awk_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_awk_head",
                command: "pipe",
                description: "cat output piped to awk and head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_awk_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_awk_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_awk_sort",
                command: "pipe",
                description: "cat output piped to awk and sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_awk_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_awk_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_awk_sort_uniq_wc",
                command: "pipe",
                description: "cat output piped to awk, sort, uniq, and wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_awk_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_awk_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_awk_xargs_wc",
                command: "pipe",
                description: "cat output piped to awk and xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_awk_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_awk_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_awk_first_grep_tail",
                command: "pipe",
                description: "cat file piped through awk first-field and literal grep to tail",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_awk_first_grep_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_awk_first_grep_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_awk_first_grep_sort_xargs_wc",
                command: "pipe",
                description:
                    "cat path-list file piped through awk first-field, literal grep, sort, and xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_awk_first_grep_sort_xargs_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_awk_first_grep_sort_xargs_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_cat_awk_sort_xargs",
                command: "pipe",
                description: "cat output piped to awk, sort, and xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &cat_awk_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &cat_awk_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_all_xargs",
                command: "pipe",
                description: "find all regular files piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_all_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_all_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_all_xargs_sort",
                command: "pipe",
                description: "find all regular files piped to xargs wc -l and sorted",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_all_xargs_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_all_xargs_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_all_xargs_echo",
                command: "pipe",
                description: "find all regular files piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_all_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_all_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_all_xargs_default",
                command: "pipe",
                description: "find all regular files piped to default xargs",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_all_xargs_default_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_all_xargs_default_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_all_wc",
                command: "pipe",
                description: "find all regular files piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_all_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_all_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_all_head",
                command: "pipe",
                description: "find all regular files piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_all_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_all_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_all_tail",
                command: "pipe",
                description: "find all regular files piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_all_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_all_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_all_sort_wc",
                command: "pipe",
                description: "find all regular files piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_all_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_all_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_maxdepth_wc",
                command: "pipe",
                description: "find maxdepth one regular files piped to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_maxdepth_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_maxdepth_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_maxdepth_head",
                command: "pipe",
                description: "find maxdepth one regular files piped to head",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_maxdepth_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_maxdepth_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_maxdepth_grep_wc",
                command: "pipe",
                description: "find maxdepth one regular files piped through literal grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_maxdepth_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_maxdepth_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_maxdepth_xargs_echo",
                command: "pipe",
                description: "find maxdepth one regular files piped to xargs echo",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_maxdepth_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_maxdepth_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_maxdepth_two_sort_tail",
                command: "pipe",
                description: "find maxdepth two regular files piped through sort to tail",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_maxdepth_two_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_maxdepth_two_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_maxdepth_two_name_grep_wc",
                command: "pipe",
                description: "find maxdepth two named regular files piped through grep to wc -l",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_maxdepth_two_name_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_maxdepth_two_name_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_xargs",
                command: "pipe",
                description: "find results piped to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_xargs_echo",
                command: "pipe",
                description: "find results piped to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_xargs_default",
                command: "pipe",
                description: "find results piped to default xargs",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_xargs_default_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_xargs_default_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_grep_xargs_echo",
                command: "pipe",
                description: "find results piped through literal grep to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_grep_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_grep_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_grep_xargs_wc",
                command: "pipe",
                description: "find results piped through literal grep to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_grep_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_grep_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_grep_wc",
                command: "pipe",
                description: "find results piped through literal grep to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_grep_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_grep_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_grep_head",
                command: "pipe",
                description: "find results piped through literal grep to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_grep_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_grep_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_grep_sort_uniq_wc",
                command: "pipe",
                description: "find results piped through literal grep, sort, and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_grep_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_grep_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_grep_sort_xargs_echo",
                command: "pipe",
                description: "find results piped through literal grep and sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_grep_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_grep_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_grep_sort_xargs_wc",
                command: "pipe",
                description: "find results piped through literal grep and sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_grep_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_grep_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_grep_sort_xargs_wc_sort",
                command: "pipe",
                description:
                    "find results piped through literal grep, sort, xargs wc -l, and sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_grep_sort_xargs_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_grep_sort_xargs_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_wc",
                command: "pipe",
                description: "find results piped to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_head",
                command: "pipe",
                description: "find results piped to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_tail",
                command: "pipe",
                description: "find results piped to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_sort",
                command: "pipe",
                description: "find results piped to sort",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_sort_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_sort_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_sort_uniq",
                command: "pipe",
                description: "find results piped through sort to uniq",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_sort_uniq_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_sort_uniq_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_sort_uniq_wc",
                command: "pipe",
                description: "find results piped through sort and uniq to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_sort_uniq_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_sort_uniq_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_sort_wc",
                command: "pipe",
                description: "find results piped through sort to wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_sort_wc_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_sort_wc_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_sort_xargs_echo",
                command: "pipe",
                description: "find results piped through sort to xargs echo",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_sort_xargs_echo_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_sort_xargs_echo_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_sort_xargs",
                command: "pipe",
                description: "find results piped through sort to xargs wc -l",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_sort_xargs_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_sort_xargs_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_sort_xargs_wc_sort_tail",
                command: "pipe",
                description: "find results piped through sort, xargs wc -l, sort, and tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_sort_xargs_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_sort_xargs_sort_tail_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_sort_head",
                command: "pipe",
                description: "find results piped through sort to head",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_sort_head_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_sort_head_pipe]),
                stdin_file: None,
            },
            Scenario {
                id: "pipe_find_sort_tail",
                command: "pipe",
                description: "find results piped through sort to tail",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &find_sort_tail_pipe]),
                original_program: "/bin/bash".to_string(),
                original_args: strings(["-c", &find_sort_tail_pipe]),
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
                id: "grep_recursive_small_takeover",
                command: "grep",
                description: "small recursive grep takeover path",
                gate: Gate::Takeover,
                expected_exit_code: 0,
                cap_args: strings(["grep", "-R", "NEEDLE", &path_string(&small_grep_root)]),
                original_program: "/usr/bin/grep".to_string(),
                original_args: strings(["-R", "NEEDLE", &path_string(&small_grep_root)]),
                stdin_file: None,
            },
            Scenario {
                id: "grep_file_literal",
                command: "grep",
                description: "single large text file, literal search",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["grep", "NEEDLE", &path_string(&sed_file)]),
                original_program: "/usr/bin/grep".to_string(),
                original_args: strings(["NEEDLE", &path_string(&sed_file)]),
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
            Scenario {
                id: "run_string_grep_file",
                command: "run",
                description: "hook string: grep one large text file, literal search",
                gate: Gate::DualWin,
                expected_exit_code: 0,
                cap_args: strings(["run", &run_grep_file]),
                original_program: "/usr/bin/grep".to_string(),
                original_args: strings(["NEEDLE", &path_string(&sed_file)]),
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
