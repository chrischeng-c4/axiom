# meter

## Brief

**Local resource measurement for agent-driven Rust development.**

`meter` is the renamed and narrowed successor to `qc`. Its job is to help an
agent see where a local run spends time and which resource signal deserves the
next action. It is not a test framework, not a runtime/environment manager, and
not a security scanner. `vat` prepares the local env/runner; `meter` measures the
run and emits agent-readable findings.

Current public scope:

- External executable measurement: cpu_time / wall_time / peak RSS, plus optional
  sampled CPU hot spots.
- Source/runtime profiling policy via `meter.toml` `level`, with shipped
  embedded phase/boundary-cost folding.
- Benchmark regression folding from saved baselines.
- Delegated test failure packaging, without replacing the test runner.
- Deterministic JSON reports and offline LLM/self-description docs.

Planned resource scope:

- Leak growth over time (peak RSS per run shipped via capture vitals).
- IO, disk, network, and GPU attribution.
- AST-assisted probe placement so agents can request finer measurement without
  hand-editing product code.

Security/audit/fuzz code still exists as carried legacy internals from the old
`qc` shape. It is not part of the public `meter` capability surface.

## Mental Model

```text
vat run
  prepares env + runner + data
  streams key checkpoints
  runs the workload
        |
        v
meter run / measure / profile / bench / test
  delegates tests when needed
  samples or folds resource evidence
  prints one MeterReport JSON document
        |
        v
agent
  reads findings[].evidence + findings[].invoke
  fixes or asks for a smaller measurement target
```

`meter` has two complementary modes:

- **Measure mode** observes from outside the workload. It can run an executable
  or cargo target and record simple process vitals, with optional stack sampling.
- **Profile mode** consumes source/runtime-aware data. The shipped path folds
  measurement data emitted by code that already uses `meter` APIs, such as
  `Profiler`, `BoundaryTracer`, and `Benchmarker`.

The ideal future shape is mixed: AST finds reasonable instrumentation points,
generated probes collect data, and `measure` / `profile` fold everything into
one report.

## Command Split

| Command | Target | Uses | Output |
|---|---|---|---|
| `meter measure <target>` | Binary, executable path, or command on `PATH` | External observation when the target can only be run as a process. | `vital` findings for cpu/wall/RSS; `hotspot` findings and `.meter/*.collapsed` at `--level sample`. |
| `meter measure --bin/--example/--bench/--exec <target>` | Cargo target or explicit executable path | Same external observation, with cargo build/target resolution when needed. | Same `vital` / `hotspot` findings. |
| `meter profile --phases <file>` | Serialized `PhaseBreakdown` emitted by meter APIs | Embedded profiling data from code that can instrument itself. | `boundary_cost` findings. |
| `meter profile <source-target>` | Future RS/TS/PY source/runtime target | Reserved for AST/runtime-assisted auto-instrumentation. | Clear unsupported message until probe injection is wired. |

`meter.toml` is intentionally narrow. It may carry profile policy such as
`level = "vitals"` or `level = "sample"`, but it does not carry project resource
gates. Thresholds such as max RSS, p99, qps, or data-size policy belong in the
project's EC/arena/rig/vat configuration.

## Data Location

`meter` writes local measurement artifacts under `.meter/` in the current
workspace:

- `.meter/last-report.json` is the best-effort persisted `MeterReport` for
  `meter report`.
- `.meter/<target>.collapsed` is written by `meter measure --level sample` when
  folded stack samples are available.

These files are local evidence/cache artifacts, not source of truth. The
machine-readable stdout report is the primary agent contract for each run.


## Capabilities

Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Runtime Resource Attribution | #3 | implemented | verified | smoke | ready | measure/profile, phase boundary cost, benchmark regression |
| Agent Use First CLI | - | implemented | verified | smoke | ready | JSON-default CLI and offline LLM/spec contract |
| Legacy Carried Internals | #3 | retired | verified | smoke | retired | Old qc-era modules retained for compatibility, not public meter capability |

### Runtime Resource Attribution

ID: runtime-resource-attribution
Type: DeveloperTool
Root WI: #3
Status: verified
Required Verification: smoke
Promise:
meter emits ranked runtime/resource findings so an agent can identify where time goes and catch benchmark regressions outside ordinary unit tests.
Gate Inventory:
- `cargo run -p meter-cli --bin meter -- measure --exec /bin/ls --compact`; `cargo run -p meter-cli --bin meter -- profile --phases projects/meter/tests/fixtures/profile_phase_breakdown.json`; `cargo test -p meter performance::profiler`; `cargo test -p meter benchmark::`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Profile phase boundary-cost report | epic | - | implemented | verified | smoke | `cargo run -p meter-cli --bin meter -- profile --phases projects/meter/tests/fixtures/profile_phase_breakdown.json` |
| Embedded profiler API | epic | - | implemented | verified | smoke | `cargo test -p meter performance::profiler` |
| Benchmark regression API | epic | - | implemented | verified | smoke | `cargo test -p meter benchmark::` |
| Capture vitals and measurement contract | change | #3 | implemented | verified | smoke | `cargo test -p meter capture::vitals` |

Shipped behavior:

- `meter measure <target>|--bin|--example|--bench|--exec <target>` measures an
  external executable or cargo target: level `vitals` (default) emits
  `Finding{kind:vital}`
  (cpu_time_ms / wall_time_ms / peak_rss_bytes via wait4+rusage, no sampler);
  level `sample` adds ranked `Finding{kind:hotspot}` evidence plus a
  `.meter/<target>.collapsed` artifact. The window lasts until the child exits
  (`--duration-cap` bounds it; `--drive <cmd>` runs an opaque driver whose exit
  ends the window — meter never generates load). `measure` does not read
  project gates; threshold policy belongs to the project EC/arena/rig/vat layer.
- `meter profile --phases <file>` reads a serialized `PhaseBreakdown` and emits
  `Finding{kind:boundary_cost}` without sampler privileges.
- `meter profile <source-target>` is reserved for source/runtime-aware
  auto-instrumentation; direct RS/TS/PY probe injection is not wired yet.
- `meter bench --target <crate> --baseline <file>` folds benchmark regressions
  into `Finding{kind:regression}` and exits 2 for medium-or-worse regressions.
- Embedded APIs provide phase timing, boundary tracing, benchmark stats, and
  baseline comparison for code that can already emit measurement data.

Known limits:

- IO, disk, GPU, network, and leak detection are not public signals yet.
- Source auto-discovery/probe injection is not wired; `profile` currently needs
  embedded phase data.

### Agent Use First CLI

ID: agent-use-first-cli
Type: DeveloperTool
Root WI: -
Status: verified
Required Verification: smoke
Promise:
meter's default CLI output is deterministic JSON with machine-readable findings, next actions, environment, completion, and delegated-run exit semantics for agents.
Gate Inventory:
- `cargo run -p meter-cli --bin meter -- spec --json-schema --compact`; `cargo run -p meter-cli --bin meter -- spec --catalog --compact`; `cargo test -p meter report::`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| JSON-default report envelope and findings | epic | - | implemented | verified | smoke | `cargo test -p meter report::` |
| Offline schema and catalog self-description | epic | - | implemented | verified | smoke | `cargo run -p meter-cli --bin meter -- spec --catalog --compact` |
| Delegated runner exit-code contract | epic | - | implemented | verified | smoke | `cargo test -p meter report::builder::tests::forward_exit_overrides_natural_code` |
| LLM usage guide | epic | - | implemented | verified | smoke | `cargo run -p meter-cli --bin meter -- llm guide` |

Shipped behavior:

- JSON is the default stdout for populator verbs.
- Diagnostics and `--human` summaries go to stderr.
- `schema_version` is `meter.report/1`.
- `meter spec --json-schema` and `meter spec --catalog` are offline.
- `meter llm guide` and `meter llm recipes` tell an agent how to use meter
  without spending tokens on general help output.
- `meter test` delegates to `cargo nextest` or `cargo test` and forwards the
  child exit code.
- `meter run` delegates test by default and folds opt-in bench/profile findings
  into one worst-wins report.

### Legacy Carried Internals

ID: legacy-carried-internals
Root WI: #3
Status: retired
Required Verification: smoke
Promise:
meter retains old qc-era modules only so dependent crates and tests continue to build while the public meter surface narrows.
Gate Inventory:
- `cargo test -p meter`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Cargo audit advisory detection | epic | - | out_of_scope | verified | smoke | `cargo test -p meter --test audit_trust_bug` |
| Seeded fuzz and injection finding generation | epic | - | out_of_scope | verified | smoke | `cargo test -p meter security::` |
| Agent-eval and legacy reporter internals | epic | - | out_of_scope | verified | smoke | `cargo test -p meter` |
| Stress residue prune | change | #3 | implemented | verified | smoke | `cargo test -p meter` |

These modules are intentionally not listed in `meter --help`, `meter spec
--catalog`, or `meter llm recipes`. They are compatibility code until a later
prune or separate product decision.


## CLI

All public verbs ship through the `meter-cli` crate.

```bash
cargo run -p meter-cli --bin meter -- llm guide
cargo run -p meter-cli --bin meter -- run --target .
cargo run -p meter-cli --bin meter -- measure --example profile_target --duration 3
cargo run -p meter-cli --bin meter -- profile --phases projects/meter/tests/fixtures/profile_phase_breakdown.json
cargo run -p meter-cli --bin meter -- bench --target . --baseline baseline.json
cargo run -p meter-cli --bin meter -- test -- -p meter --lib
cargo run -p meter-cli --bin meter -- report
cargo run -p meter-cli --bin meter -- spec --catalog --compact
```

Public verbs:

- `test` delegates and forwards the child runner exit.
- `bench` delegates `cargo bench` and folds a serialized regression baseline.
- `measure` records external executable vitals and optional CPU stack samples.
- `profile` folds serialized phase data today; source auto-instrumentation is
  the reserved direction.
- `run` composes test plus opt-in bench/profile into one report.
- `report` and `state` re-project `.meter/last-report.json`.
- `spec` emits schema/catalog data.
- `llm` emits the guide or machine recipes.

## Report Contract

Every populator report is a `MeterReport`:

- `status`, `clean`, `exit_code`, and `terminal` are machine-readable.
- `findings[]` carries `id`, `severity`, `kind`, `remediation`, `invoke`, and
  structured `evidence`.
- Public finding kinds are `vital`, `hotspot`, `boundary_cost`, `regression`,
  and `test_failure`.
- `completion.missing` lists skipped or un-driven sub-verbs so an agent can see
  coverage gaps.
- `.meter/last-report.json` is best-effort persisted for `meter report`.

Exit codes:

- `0`: clean
- `1`: findings
- `2`: regression
- `3`: usage
- `4`: missing tool
- `5`: IO or spawn failure

For `meter test`, the process exit code is the delegated child exit code.

## Library Usage

```toml
[dev-dependencies]
meter = { path = "../meter" }
```

```rust
use meter::performance::profiler::Profiler;

let mut profiler = Profiler::new(Default::default());
// mark phases in code that already opts into embedded measurement
let result = profiler.finish();
```

## Known Gaps

- IO, disk, network, GPU, and leak detection are not wired into the public CLI yet.
- AST-assisted instrumentation is planned but not implemented.
- The `meter-cli` crate registers a `CliModule`, but no aggregating `cclab` host
  binary exposes `cclab meter <verb>` in-tree.
- Legacy modules from the old `qc` shape remain: `agent_eval`, `security`,
  `capture::audit`, `capture::fuzz`, `http_server`, `ts_runner`, `parametrize`,
  `fixtures`, `hooks`, `plugin`, and the older reporter envelope. They are
  carried internals, not public meter capability.
- `crates/cclab-qc-mamba` still keeps its historical crate name while depending
  on `meter` through Cargo's `package = "meter"` alias.
