---
id: projects-meter-src-capture-sampler-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/capture/sampler.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/capture/sampler.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FoldedStack` | projects/meter/src/capture/sampler.rs | struct | pub | 35 |  |
| `SampleError` | projects/meter/src/capture/sampler.rs | enum | pub | 79 |  |
| `SampleRun` | projects/meter/src/capture/sampler.rs | struct | pub | 64 |  |
| `Target` | projects/meter/src/capture/sampler.rs | enum | pub | 105 |  |
| `label` | projects/meter/src/capture/sampler.rs | function | pub | 120 | label(&self) -> String |
| `leaf` | projects/meter/src/capture/sampler.rs | function | pub | 50 | leaf(&self) -> Option<&str> |
| `new` | projects/meter/src/capture/sampler.rs | function | pub | 45 | new(frames: Vec<String>, count: u64) -> Self |
| `parse_perf_script` | projects/meter/src/capture/sampler.rs | function | pub | 649 | parse_perf_script(text: &str) -> Vec<FoldedStack> |
| `parse_perf_script` | projects/meter/src/capture/sampler.rs | function | pub | 687 | parse_perf_script(_text: &str) -> Vec<FoldedStack> |
| `parse_sample_report` | projects/meter/src/capture/sampler.rs | function | pub | 461 | parse_sample_report(report: &str) -> Vec<FoldedStack> |
| `resolve_target_exec` | projects/meter/src/capture/sampler.rs | function | pub | 363 | resolve_target_exec(target: &Target) -> Result<PathBuf, SampleError> |
| `sample_target` | projects/meter/src/capture/sampler.rs | function | pub | 138 | sample_target(     target: &Target,     extra_args: &[String],     duration_secs: u64,     hz: Option<u64>, ) -> Result<SampleRun, SampleError> |
| `spawn_exec` | projects/meter/src/capture/sampler.rs | function | pub | 433 | spawn_exec(     exec: &std::path::Path,     extra_args: &[String], ) -> Result<Child, SampleError> |
| `to_folded_line` | projects/meter/src/capture/sampler.rs | function | pub | 55 | to_folded_line(&self) -> String |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Platform stack sampler — the net-new C1 capture-mode producer.
//!
//! `meter` profiles a workload by spawning it as a child, attaching the platform
//! stack sampler to the child PID, and FOLDING the resulting call tree into
//! [`FoldedStack`]s (root -> leaf frame lists with leaf sample counts). Nothing
//! in the engine fed [`FlamegraphData::add_stack`] before; this module is that
//! missing producer.
//!
//! Backends:
//! - macOS: `/usr/bin/sample <pid> <secs> <interval_ms> -file <tmp> -mayDie`.
//!   Always present, needs no entitlement. Its report is an indented "Call
//!   graph:" tree with INCLUSIVE sample counts per node; [`parse_sample_report`]
//!   folds it into root->leaf stacks with LEAF (self) counts.
//! - Linux: `perf record -F <hz> -g -- <child>` then `perf script`, folded by
//!   collapsing `perf script` frames. macOS is the path that must work here.
//!
//! If no backend is available the sampler returns [`SampleError::NoBackend`],
//! which the dispatch layer maps to `ToolError(4)` with an install/availability
//! hint — NEVER a fake-clean result.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// One folded call stack: a root->leaf list of frame symbols plus the number of
/// samples whose LEAF (innermost frame) is this stack's leaf.
///
/// `frames` is ROOT-FIRST (frames[0] is the outermost caller, the last element
/// is the innermost/leaf symbol). `count` is the number of samples that
/// terminated at this leaf (self samples), which is exactly the folded-stacks /
/// inferno convention.
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
pub struct FoldedStack {
    /// Root-first frame symbols (last = leaf).
    pub frames: Vec<String>,
    /// Number of samples whose leaf is this stack's leaf.
    pub count: u64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
impl FoldedStack {
    /// Construct a folded stack from root-first `frames` and a self `count`.
    pub fn new(frames: Vec<String>, count: u64) -> Self {
        Self { frames, count }
    }

    /// The leaf (innermost) symbol, if any.
    pub fn leaf(&self) -> Option<&str> {
        self.frames.last().map(|s| s.as_str())
    }

    /// Render as a single inferno folded line: `a;b;c count`.
    pub fn to_folded_line(&self) -> String {
        format!("{} {}", self.frames.join(";"), self.count)
    }
}

/// The result of a sampling run: the folded stacks plus the backend used and the
/// effective sampling rate (Hz) so the fold step can convert samples -> ns.
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
pub struct SampleRun {
    /// Folded root->leaf stacks with leaf (self) counts.
    pub stacks: Vec<FoldedStack>,
    /// The backend that produced the stacks (`macos-sample` / `linux-perf`).
    pub backend: String,
    /// Effective sampling rate in Hz (samples per second) used to map
    /// `samples -> ns` in the fold step.
    pub effective_hz: f64,
    /// The argv of the sampler invocation, for the `RunnerRecord`.
    pub command: Vec<String>,
}

/// Errors a sampling run can surface.
#[derive(Debug)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
pub enum SampleError {
    /// No stack sampler is available on this platform (maps to `ToolError(4)`).
    NoBackend(String),
    /// The target child could not be spawned/built (maps to `ToolError(5)`).
    Spawn(String),
    /// The sampler ran but its report could not be read/parsed (`ToolError(4)`).
    Sampler(String),
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
impl std::fmt::Display for SampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleError::NoBackend(m) => write!(f, "no stack sampler backend available: {m}"),
            SampleError::Spawn(m) => write!(f, "could not spawn target: {m}"),
            SampleError::Sampler(m) => write!(f, "sampler failed: {m}"),
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
impl std::error::Error for SampleError {}

/// How to launch the workload to be sampled.
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
pub enum Target {
    /// `cargo run --bin <name> [-- <args>]`.
    Bin(String),
    /// `cargo run --example <name> [-- <args>]`.
    Example(String),
    /// `cargo bench --bench <name>` (note: benches re-build; sampling a bench is
    /// best-effort because criterion harnesses fork).
    Bench(String),
    /// A pre-built executable path, run directly.
    Exec(PathBuf),
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
impl Target {
    /// A short human label for the target (used as `MeterReport.target`).
    pub fn label(&self) -> String {
        match self {
            Target::Bin(n) => format!("bin:{n}"),
            Target::Example(n) => format!("example:{n}"),
            Target::Bench(n) => format!("bench:{n}"),
            Target::Exec(p) => format!("exec:{}", p.display()),
        }
    }
}

/// Sample `target` for `duration_secs`, optionally pinning the sampling interval
/// (ms). `hz`, when given, overrides the default interval (interval_ms =
/// 1000/hz). Returns folded stacks + the backend + effective Hz.
///
/// macOS: builds the target first (so `cargo run` build output does not eat the
/// sampling window), spawns the child, samples its PID with `/usr/bin/sample`,
/// waits for the child, then parses the report. No backend => `NoBackend`.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
pub fn sample_target(
    target: &Target,
    extra_args: &[String],
    duration_secs: u64,
    hz: Option<u64>,
) -> Result<SampleRun, SampleError> {
    if cfg!(target_os = "macos") {
        sample_macos(target, extra_args, duration_secs, hz)
    } else if cfg!(target_os = "linux") {
        sample_linux(target, extra_args, duration_secs, hz)
    } else {
        Err(SampleError::NoBackend(format!(
            "platform `{}` has no supported stack sampler (macOS `sample` / Linux `perf`)",
            std::env::consts::OS
        )))
    }
}

/// Default sampling interval in ms when no `--hz` is given (macOS default is
/// 1ms, but 4ms is plenty for hot-spot ranking and far cheaper).
const DEFAULT_INTERVAL_MS: u64 = 4;

/// macOS backend: spawn the child, sample its PID with `/usr/bin/sample`.
#[cfg(target_os = "macos")]
fn sample_macos(
    target: &Target,
    extra_args: &[String],
    duration_secs: u64,
    hz: Option<u64>,
) -> Result<SampleRun, SampleError> {
    use std::io::Read;

    // /usr/bin/sample must exist.
    if !std::path::Path::new("/usr/bin/sample").exists() {
        return Err(SampleError::NoBackend(
            "/usr/bin/sample not found (expected on macOS)".to_string(),
        ));
    }

    // Build cargo targets and resolve the produced executable so the SAMPLED
    // PID is the workload itself (not a `cargo run` parent that forks it).
    let exec_path = resolve_target_exec(target)?;

    let interval_ms = match hz {
        Some(h) if h > 0 => (1000 / h).max(1),
        _ => DEFAULT_INTERVAL_MS,
    };
    let effective_hz = 1000.0 / interval_ms as f64;

    // Spawn the workload directly. stdout/stderr are discarded so they never
    // pollute meter's own JSON-on-stdout document.
    let mut child = spawn_exec(&exec_path, extra_args)?;
    let pid = child.id();

    // Temp file for the sample report.
    let report_path = std::env::temp_dir().join(format!("meter-sample-{pid}.txt"));

    let argv = vec![
        "/usr/bin/sample".to_string(),
        pid.to_string(),
        duration_secs.to_string(),
        interval_ms.to_string(),
        "-file".to_string(),
        report_path.display().to_string(),
        "-mayDie".to_string(),
    ];

    let sample_output = Command::new("/usr/bin/sample").args(&argv[1..]).output();

    // The child may still be running (sampler only observes it); make sure it
    // does not outlive us regardless of how sampling went.
    let _ = child.kill();
    let _ = child.wait();

    match sample_output {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let _ = std::fs::remove_file(&report_path);
            return Err(SampleError::Sampler(format!(
                "/usr/bin/sample exited {} for pid {pid}; stderr: {}; stdout: {}",
                output.status,
                process_output_preview(&output.stderr),
                process_output_preview(&output.stdout)
            )));
        }
        Err(e) => {
            let _ = std::fs::remove_file(&report_path);
            return Err(SampleError::Sampler(format!(
                "/usr/bin/sample failed to run: {e}"
            )));
        }
    }

    // Read + parse the report.
    let mut content = String::new();
    match std::fs::File::open(&report_path).and_then(|mut f| f.read_to_string(&mut content)) {
        Ok(_) => {}
        Err(e) => {
            return Err(SampleError::Sampler(format!(
                "could not read sample report `{}`: {e}",
                report_path.display()
            )));
        }
    }
    let _ = std::fs::remove_file(&report_path);

    let stacks = parse_sample_report(&content);
    if stacks.is_empty() {
        return Err(SampleError::Sampler(format!(
            "sample report contained no call-graph stacks (target may have exited before sampling \
             attached, or ran too briefly); report was {} bytes",
            content.len()
        )));
    }

    Ok(SampleRun {
        stacks,
        backend: "macos-sample".to_string(),
        effective_hz,
        command: argv,
    })
}

#[cfg(not(target_os = "macos"))]
fn sample_macos(
    _target: &Target,
    _extra_args: &[String],
    _duration_secs: u64,
    _hz: Option<u64>,
) -> Result<SampleRun, SampleError> {
    Err(SampleError::NoBackend("not macOS".to_string()))
}

/// Linux backend: `perf record -F <hz> -g -- <child>` then `perf script`,
/// collapsed into folded stacks. Best-effort; macOS is the supported path here.
#[cfg(target_os = "linux")]
fn sample_linux(
    target: &Target,
    extra_args: &[String],
    _duration_secs: u64,
    hz: Option<u64>,
) -> Result<SampleRun, SampleError> {
    // perf must be present.
    let perf_ok = Command::new("perf")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !perf_ok {
        return Err(SampleError::NoBackend(
            "`perf` not found (install linux-tools / linux-perf)".to_string(),
        ));
    }
    // Resolve the built executable so perf records the workload directly (not a
    // `cargo run` parent).
    let exec_path = resolve_target_exec(target)?;

    let freq = hz.unwrap_or(250).max(1);
    let effective_hz = freq as f64;

    let perf_data = std::env::temp_dir().join(format!("meter-perf-{}.data", std::process::id()));

    let mut record = Command::new("perf");
    record
        .args(["record", "-F", &freq.to_string(), "-g", "-o"])
        .arg(&perf_data)
        .arg("--")
        .arg(&exec_path)
        .args(extra_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let argv = vec![
        "perf".to_string(),
        "record".to_string(),
        "-F".to_string(),
        freq.to_string(),
        "-g".to_string(),
    ];
    record
        .status()
        .map_err(|e| SampleError::Sampler(format!("perf record failed: {e}")))?;

    let script = Command::new("perf")
        .arg("script")
        .arg("-i")
        .arg(&perf_data)
        .output()
        .map_err(|e| SampleError::Sampler(format!("perf script failed: {e}")))?;
    let _ = std::fs::remove_file(&perf_data);

    let text = String::from_utf8_lossy(&script.stdout);
    let stacks = parse_perf_script(&text);
    if stacks.is_empty() {
        return Err(SampleError::Sampler(
            "perf script produced no stacks".to_string(),
        ));
    }
    Ok(SampleRun {
        stacks,
        backend: "linux-perf".to_string(),
        effective_hz,
        command: argv,
    })
}

#[cfg(not(target_os = "linux"))]
fn sample_linux(
    _target: &Target,
    _extra_args: &[String],
    _duration_secs: u64,
    _hz: Option<u64>,
) -> Result<SampleRun, SampleError> {
    Err(SampleError::NoBackend("not Linux".to_string()))
}

/// Build a cargo target and resolve the EXECUTABLE it produced, so the sampled
/// PID is the workload itself (not a `cargo run` parent that forks the real
/// binary — sampling the parent yields only idle-wait stacks). Exec targets
/// resolve to their path directly.
///
/// Uses `cargo build --message-format=json` and reads the last
/// `compiler-artifact` message that carries an `executable` path for the target.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
pub(crate) fn resolve_target_exec(target: &Target) -> Result<PathBuf, SampleError> {
    let mut build = Command::new("cargo");
    match target {
        Target::Bin(name) => {
            build.args(["build", "--message-format=json", "--bin", name]);
        }
        Target::Example(name) => {
            build.args(["build", "--message-format=json", "--example", name]);
        }
        Target::Bench(name) => {
            build.args(["build", "--message-format=json", "--bench", name]);
        }
        Target::Exec(path) => {
            if !path.exists() {
                return Err(SampleError::Spawn(format!(
                    "exec target `{}` does not exist",
                    path.display()
                )));
            }
            return Ok(path.clone());
        }
    }
    // Capture stdout (JSON artifact stream); inherit stderr so build progress is
    // visible but never lands on meter's stdout.
    let output = build
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| SampleError::Spawn(format!("cargo build failed to launch: {e}")))?;
    if !output.status.success() {
        return Err(SampleError::Spawn(format!(
            "cargo build for `{}` exited non-zero",
            target.label()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let exec = last_executable_artifact(&stdout).ok_or_else(|| {
        SampleError::Spawn(format!(
            "cargo build for `{}` produced no executable artifact",
            target.label()
        ))
    })?;
    Ok(PathBuf::from(exec))
}

/// Scan `cargo build --message-format=json` stdout for the last
/// `compiler-artifact` message carrying a non-null `executable` path.
fn last_executable_artifact(json_stream: &str) -> Option<String> {
    let mut found: Option<String> = None;
    for line in json_stream.lines() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v.get("reason").and_then(|r| r.as_str()) == Some("compiler-artifact") {
            if let Some(exe) = v.get("executable").and_then(|e| e.as_str()) {
                found = Some(exe.to_string());
            }
        }
    }
    found
}

/// Spawn the workload executable directly (stdout/stderr discarded) so the
/// sampled PID is the workload.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
pub(crate) fn spawn_exec(
    exec: &std::path::Path,
    extra_args: &[String],
) -> Result<Child, SampleError> {
    let mut cmd = Command::new(exec);
    cmd.args(extra_args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.spawn()
        .map_err(|e| SampleError::Spawn(format!("could not spawn `{}`: {e}", exec.display())))
}

// ============================================================================
// macOS `sample` call-graph parser
// ============================================================================

/// Parse a macOS `sample` report into folded root->leaf stacks with LEAF (self)
/// counts.
///
/// The report's "Call graph:" section is an indented tree where each node line
/// is `<indent><inclusive_count> <frame...>`. Indentation is 2 spaces per depth
/// level. The INCLUSIVE count at a node is the number of samples in which that
/// node appeared on the stack at that depth. A node's SELF (leaf) count is
/// `inclusive_count - sum(child inclusive_counts)`; any node with a positive
/// self count contributes a folded stack (root->that node) carrying that self
/// count. This yields exactly the leaf-sample folding inferno expects.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
pub fn parse_sample_report(report: &str) -> Vec<FoldedStack> {
    // Collect raw (depth, count, symbol) nodes from the Call graph section.
    let mut nodes: Vec<Node> = Vec::new();
    let mut in_graph = false;
    for line in report.lines() {
        if line.starts_with("Call graph:") {
            in_graph = true;
            continue;
        }
        if !in_graph {
            continue;
        }
        // The call-graph section ends at the totals block ("Total number in
        // stack ...") or "Binary Images:". Blank lines may separate per-thread
        // subtrees, so they do not terminate the section.
        let trimmed = line.trim_start();
        if trimmed.starts_with("Total number in stack") || trimmed.starts_with("Binary Images:") {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }
        if let Some(node) = parse_node_line(line) {
            nodes.push(node);
        }
    }

    fold_nodes(&nodes)
}

/// A single parsed call-graph node: indentation depth, inclusive sample count,
/// and the cleaned symbol name.
struct Node {
    depth: usize,
    count: u64,
    symbol: String,
}

/// Parse one call-graph line into a [`Node`]: tree-prefix -> depth (2 columns
/// per level), then the leading integer count, then the symbol up to the
/// ` (in <module>)` / ` + offset` decoration.
///
/// Once a subtree branches, macOS `sample` switches from plain 2-space
/// indentation to tree-drawing glyph columns (`+ `, `! `, `: `, `| `, `  `),
/// e.g. `  + ! : | 574 frame ...`. Both encodings are 2 columns per depth
/// level, so depth = prefix-width / 2 where the prefix is every leading
/// space or glyph column before the sample count.
fn parse_node_line(line: &str) -> Option<Node> {
    // Measure the tree prefix: spaces plus the `+ ! : |` glyphs that `sample`
    // draws once siblings appear. The count token is the first run of digits
    // after that prefix. The shallowest node (the thread line) sits at
    // indent 4.
    let indent = line
        .find(|c: char| !matches!(c, ' ' | '+' | '!' | ':' | '|'))
        .unwrap_or(line.len());
    let rest = &line[indent..];
    // The first whitespace-separated token is the inclusive sample count.
    let mut parts = rest.splitn(2, char::is_whitespace);
    let count_tok = parts.next()?;
    let count: u64 = count_tok.parse().ok()?;
    let after_count = parts.next().unwrap_or("").trim_start();
    let symbol = clean_symbol(after_count);
    if symbol.is_empty() {
        return None;
    }
    // Depth in half-indents; the shallowest meaningful node is at indent 4
    // (depth 0). Guard against weird indents.
    let depth = indent / 2;
    Some(Node {
        depth,
        count,
        symbol,
    })
}

/// Clean a raw frame description into a stable symbol name. Strips the
/// ` (in <module>)`, ` + <offset>`, and `[<addresses>]` decorations and the
/// thread/queue annotations, leaving the function name (or thread label).
fn clean_symbol(raw: &str) -> String {
    let mut s = raw.trim();
    // Drop trailing address bracket: `... [0x...]`.
    if let Some(idx) = s.rfind("  [0x") {
        s = &s[..idx];
    } else if let Some(idx) = s.rfind(" [0x") {
        s = &s[..idx];
    }
    // Drop ` + <offset>` (and any following `,offset,...`).
    if let Some(idx) = s.find(" + ") {
        s = &s[..idx];
    }
    // Drop ` (in <module>)`.
    if let Some(idx) = s.find("  (in ") {
        s = &s[..idx];
    } else if let Some(idx) = s.find(" (in ") {
        s = &s[..idx];
    }
    // Thread header lines look like `Thread_12345   DispatchQueue_1: ...`.
    // Keep just the thread token so the root frame is stable across runs.
    if let Some(first) = s.split_whitespace().next() {
        if first.starts_with("Thread_") {
            return first.to_string();
        }
    }
    s.trim().to_string()
}

/// Fold a flat list of depth-ordered nodes (pre-order traversal of the call
/// tree) into root->leaf folded stacks with self counts.
fn fold_nodes(nodes: &[Node]) -> Vec<FoldedStack> {
    // Normalize depths so the shallowest node is depth 0, keeping relative order.
    let min_depth = nodes.iter().map(|n| n.depth).min().unwrap_or(0);

    // Walk the pre-order list maintaining a stack of (depth, symbol, count).
    // For each node we know the path from root via the ancestor stack. We sum
    // each node's direct children inclusive counts to recover self counts.
    let mut path: Vec<(usize, String)> = Vec::new();
    let mut children_sum: Vec<u64> = Vec::new(); // parallel to `path`: child-count accumulator
    let mut out: Vec<FoldedStack> = Vec::new();

    // We need self counts; process by emitting a node's self stack only once all
    // its children are seen. Easiest: do a second pass computing, for each node
    // index, the sum of immediate children's counts using depth structure.
    let norm: Vec<(usize, u64, &str)> = nodes
        .iter()
        .map(|n| (n.depth - min_depth, n.count, n.symbol.as_str()))
        .collect();

    let self_counts = self_counts(&norm);

    for (i, (depth, count, symbol)) in norm.iter().enumerate() {
        // Maintain the ancestor path: pop until the top is our parent (depth-1).
        while let Some(&(d, _)) = path.last() {
            if d >= *depth {
                path.pop();
                children_sum.pop();
            } else {
                break;
            }
        }
        path.push((*depth, symbol.to_string()));
        children_sum.push(0);
        let _ = (count, &mut children_sum); // children_sum retained for clarity

        let self_count = self_counts[i];
        if self_count > 0 {
            let frames: Vec<String> = path.iter().map(|(_, s)| s.clone()).collect();
            out.push(FoldedStack::new(frames, self_count));
        }
    }

    out
}

/// Compute the self (leaf) count for each node in a pre-order depth list:
/// `self = inclusive - sum(immediate children inclusive)`.
fn self_counts(norm: &[(usize, u64, &str)]) -> Vec<u64> {
    let mut selfc: Vec<u64> = norm.iter().map(|(_, c, _)| *c).collect();
    // For each node, find its immediate children (the contiguous run of nodes
    // deeper by exactly one level until depth drops back to <= this depth) and
    // subtract their inclusive counts.
    for i in 0..norm.len() {
        let (depth_i, _, _) = norm[i];
        let mut j = i + 1;
        while j < norm.len() {
            let (depth_j, count_j, _) = norm[j];
            if depth_j <= depth_i {
                break; // left this subtree
            }
            if depth_j == depth_i + 1 {
                // immediate child
                selfc[i] = selfc[i].saturating_sub(count_j);
            }
            j += 1;
        }
    }
    selfc
}

// ============================================================================
// Linux `perf script` parser
// ============================================================================

/// Parse `perf script` output into folded stacks. Each sample is a header line
/// (`comm pid ... cycles:`) followed by indented frame lines (`<addr> sym (mod)`),
/// innermost-first; a blank line separates samples. We reverse to root-first and
/// count one self sample per leaf stack.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
#[cfg(target_os = "linux")]
pub(crate) fn parse_perf_script(text: &str) -> Vec<FoldedStack> {
    use std::collections::HashMap;
    let mut counts: HashMap<Vec<String>, u64> = HashMap::new();
    let mut current: Vec<String> = Vec::new();
    let flush = |current: &mut Vec<String>, counts: &mut HashMap<Vec<String>, u64>| {
        if !current.is_empty() {
            current.reverse(); // perf is leaf-first; make root-first
            *counts.entry(std::mem::take(current)).or_insert(0) += 1;
        }
    };
    for line in text.lines() {
        if line.trim().is_empty() {
            flush(&mut current, &mut counts);
            continue;
        }
        if !line.starts_with(char::is_whitespace) && !line.starts_with('\t') {
            // sample header line — boundary
            flush(&mut current, &mut counts);
            continue;
        }
        // Frame line: `    ffffffff sym+0x.. (module)`.
        let t = line.trim();
        let sym = t.split_whitespace().nth(1).unwrap_or("").to_string();
        let sym = sym.split('+').next().unwrap_or("").to_string();
        if !sym.is_empty() {
            current.push(sym);
        }
    }
    flush(&mut current, &mut counts);
    counts
        .into_iter()
        .map(|(frames, count)| FoldedStack::new(frames, count))
        .collect()
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-sampler-rs.md#source
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
pub(crate) fn parse_perf_script(_text: &str) -> Vec<FoldedStack> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPORT: &str = "\
Analysis of sampling spin (pid 123) every 1 millisecond
Call graph:
    92 Thread_1   DispatchQueue_1: com.apple.main-thread  (serial)
      92 start  (in dyld) + 2476  [0x18e]
        92 main  (in spin) + 60  [0x104]
          80 hot  (in spin) + 16  [0x104a]
          12 cold  (in spin) + 8  [0x104b]

Total number in stack (recursive counted multiple, when >=5):
Binary Images:
";

    #[test]
    fn parses_indented_call_graph_into_leaf_stacks() {
        let stacks = parse_sample_report(REPORT);
        // Two leaves: hot (80) and cold (12). start/main/thread have no self.
        assert_eq!(stacks.len(), 2);
        let hot = stacks.iter().find(|s| s.leaf() == Some("hot")).unwrap();
        assert_eq!(hot.count, 80);
        assert_eq!(hot.frames, vec!["Thread_1", "start", "main", "hot"]);
        let cold = stacks.iter().find(|s| s.leaf() == Some("cold")).unwrap();
        assert_eq!(cold.count, 12);
    }

    #[test]
    fn self_count_subtracts_children() {
        // A node whose inclusive count exceeds the sum of its children gets the
        // remainder as a self leaf stack.
        let report = "\
Call graph:
    100 Thread_1
      100 root
        60 mid
          60 leaf
        40 other

Total number in stack:
";
        let stacks = parse_sample_report(report);
        // root has self 0 (100 - (60 mid + 40 other)); mid has self 0 (60-60);
        // leaf self 60; other self 40. So two leaf stacks.
        assert_eq!(stacks.len(), 2);
        let leaf = stacks.iter().find(|s| s.leaf() == Some("leaf")).unwrap();
        assert_eq!(leaf.count, 60);
        let other = stacks.iter().find(|s| s.leaf() == Some("other")).unwrap();
        assert_eq!(other.count, 40);
    }

    #[test]
    fn node_with_self_and_children_emits_both() {
        // A node with inclusive 100 and a single child of 70 has self 30, so it
        // should emit BOTH its own self stack and the child's.
        let report = "\
Call graph:
    100 Thread_1
      100 root
        70 child

Total:
";
        let stacks = parse_sample_report(report);
        // root self = 100 - 70 = 30; child self = 70. Two stacks.
        assert_eq!(stacks.len(), 2);
        let root = stacks.iter().find(|s| s.leaf() == Some("root")).unwrap();
        assert_eq!(root.count, 30);
        assert_eq!(root.frames, vec!["Thread_1", "root"]);
        let child = stacks.iter().find(|s| s.leaf() == Some("child")).unwrap();
        assert_eq!(child.count, 70);
    }

    #[test]
    fn parses_glyph_prefixed_branches() {
        // Once a subtree branches, `sample` switches from plain 2-space
        // indentation to `+ ! : |` glyph columns. Depth stays 2 columns per
        // level across both encodings.
        let report = "\
Call graph:
    100 Thread_1   DispatchQueue_1: com.apple.main-thread  (serial)
      100 start  (in dyld) + 6992  [0x189aefe00]
        100 main  (in bench) + 964  [0x100bd6c90]
        + 70 search  (in bench) + 8696  [0x100ca1148]  storage.rs:3371
        + ! 60 eval_and  (in bench) + 1356  [0x100cbf45c]  storage.rs:4500
        + ! : 60 range_df  (in bench) + 264,516,...  [0x100c2d17c,0x100c2d278,...]  storage.rs:1406
        + ! 10 fuse  (in bench) + 924  [0x100cc2384]
        + 30 index  (in bench) + 100  [0x100cc24cc]

Total number in stack (recursive counted multiple, when >=5):
        7       _xzm_free  (in libsystem_malloc.dylib) + 0  [0x189ccb06c]
Binary Images:
";
        let stacks = parse_sample_report(report);
        let by_leaf = |name: &str| {
            stacks
                .iter()
                .find(|s| s.leaf() == Some(name))
                .unwrap_or_else(|| panic!("missing leaf `{name}`"))
        };
        // range_df is the deep leaf under search -> eval_and.
        let range = by_leaf("range_df");
        assert_eq!(range.count, 60);
        assert_eq!(
            range.frames,
            vec!["Thread_1", "start", "main", "search", "eval_and", "range_df"]
        );
        // fuse (10) is search's other child; index (30) is main's sibling child.
        assert_eq!(by_leaf("fuse").count, 10);
        assert_eq!(by_leaf("index").count, 30);
        // The totals block must not leak into the graph as fake leaves.
        assert!(stacks.iter().all(|s| s.leaf() != Some("_xzm_free")));
    }

    #[test]
    fn blank_lines_between_thread_subtrees_do_not_terminate_the_graph() {
        let report = "\
Call graph:
    60 Thread_1
      60 main
        60 hot_a

    40 Thread_2
      40 worker
        40 hot_b

Total number in stack (recursive counted multiple, when >=5):
";
        let stacks = parse_sample_report(report);
        assert_eq!(stacks.len(), 2);
        let a = stacks.iter().find(|s| s.leaf() == Some("hot_a")).unwrap();
        assert_eq!(a.frames, vec!["Thread_1", "main", "hot_a"]);
        let b = stacks.iter().find(|s| s.leaf() == Some("hot_b")).unwrap();
        assert_eq!(b.count, 40);
    }

    #[test]
    fn clean_symbol_strips_decorations() {
        assert_eq!(clean_symbol("hot  (in spin) + 16  [0x104a]"), "hot");
        assert_eq!(clean_symbol("start  (in dyld) + 2476  [0x18e]"), "start");
        assert_eq!(
            clean_symbol("Thread_38960397   DispatchQueue_1: com.apple.main-thread  (serial)"),
            "Thread_38960397"
        );
    }

    #[test]
    fn empty_report_yields_no_stacks() {
        assert!(parse_sample_report("no call graph here\n").is_empty());
    }

    #[test]
    fn folded_line_renders_root_to_leaf() {
        let s = FoldedStack::new(vec!["a".into(), "b".into(), "c".into()], 7);
        assert_eq!(s.to_folded_line(), "a;b;c 7");
    }
}

fn process_output_preview(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "<empty>".to_string();
    }

    let mut preview: String = trimmed.chars().take(2_000).collect();
    if trimmed.chars().count() > 2_000 {
        preview.push_str("...");
    }
    preview
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/capture/sampler.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/capture/sampler.rs` captured during meter full-codegen standardization.
```
