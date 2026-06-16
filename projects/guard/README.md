# guard

Security posture gate for the cclab ecosystem.

`guard` is a **first-line static security scanner**. It owns security policy
and gate semantics. It does not replace `compass`; it consumes `compass` as the
static code-intelligence engine and turns findings into one agent-readable
report (`guard.report/1`). guard does **not** integrate upward into
`vat`/`rig`/`meter`/`arena` — those are upper-layer execution tools that may
consume guard, never the reverse.

## Division of labor

```text
compass — AST, symbols, search, PDG/data-flow, generic security diagnostics
guard   — security policy/profile, findings, gate status, baseline, AW/EC integration
vat     — isolated local runner for risky checks   (may run guard; guard never drives it)
rig     — dynamic attack/e2e journeys              (may run guard; guard never drives it)
meter   — resource and DoS evidence                (may run guard; guard never drives it)
arena   — comparative security-performance budgets (may run guard; guard never drives it)
```

## Mental model

```text
guard scan .
  run compass security diagnostics across supported source/config languages
  normalize them into guard findings
  rank by policy severity
  gate only on findings absent from the accepted baseline (.guard/baseline.json)
  persist .guard/last-report.json
  print ONE guard.report/1 JSON -> exit 0 clean / 1 new findings / 3+ tool error
```

## Verbs

| Command | Effect |
|---------|--------|
| `guard scan [path]` | Scan for static security findings; gate only on findings absent from the baseline. Zero-args reads `guard.toml` |
| `guard accept [path]` | Snapshot the current findings into `.guard/baseline.json` so they stop gating |
| `guard report` | Re-project `.guard/last-report.json` without scanning |

## Scan profiles and config

| Flag | Effect |
|---|---|
| `--profile baseline-static` | Compass security diagnostics only |
| `--profile security-lint` | Security diagnostics plus security-impacting lint, including supply-chain Docker tags and SQL injection helpers |
| `--profile strict` | Security-lint profile with stricter severity normalization |
| `--no-persist` | Do not persist `.guard/last-report.json` |

A bare `guard scan` is the agent-first common path: per-run-invariant settings
(`profile`, `no_persist`, scan `paths`) live in `guard.toml`, so the CLI stays
near-zero-args. Unknown keys are ignored, so the launcher config coexists with
an aw-managed `AW-EC-TOOL` block in the same file.

```toml
# guard.toml
profile = "security-lint"
paths = ["src", "guard-cli"]
```

Guard only owns security/policy lint. General formatting, style, and
non-security lint remain outside guard.

## Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| static-security-scan | - | implemented | verified | smoke | ready | compass-backed security diagnostics normalized into `guard.report/1`; agent-first `guard scan` reads `guard.toml` |
| security-policy-profile | - | implemented | verified | smoke | ready | `guard-baseline-static/1`, `guard-security-lint/1`, and `guard-strict/1` map security diagnostics/lint into policy findings; `guard accept` baseline gates only new findings |
| security-ec-profile | - | implemented | verified | smoke | ready | AW EC/health consumes guard reports as first-class security evidence |

## AW Verification Snapshot

| Field | Value |
|---|---|
| Last verified | 2026-06-16 |
| Production readiness | ready for static security, security lint, and security baseline |
| Tech design root | `projects/guard/tech-design` |
| Source ownership | TD-first source snapshots |
| Test gate | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard -p guard-cli` |
| CLI smoke | `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist` |
| Health gate | `aw health --project guard` |
| Explicit non-goals | AST ownership, env isolation, e2e orchestration, profiling, benchmark comparison, upward evidence composition |

## Static Security Scan

| Field | Value |
|---|---|
| ID | static-security-scan |
| Root WI | - |
| Status | verified |
| Promise | guard scans source/config files with compass and emits a deterministic `guard.report/1` security findings envelope; a bare `guard scan` runs from `guard.toml`. |
| Required Verification | smoke |
| Gate Inventory | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard`; `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo run -p guard-cli --bin guard -- scan projects/guard --compact` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Compass-backed diagnostic scan | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard scan::tests::detects_javascript_eval_as_security_finding` |
| JSON report envelope | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo run -p guard-cli --bin guard -- scan projects/guard --compact` |
| Agent-first guard.toml config | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard config::tests` |

## Security Policy Profile

| Field | Value |
|---|---|
| ID | security-policy-profile |
| Root WI | - |
| Status | verified |
| Promise | guard maps compass security diagnostics and security-impacting lint into policy severities, remediation, locations, and agent prompts; an accepted baseline suppresses known findings so the gate fires only on new ones. |
| Required Verification | smoke |
| Gate Inventory | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard`; `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard-cli` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Baseline static policy | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard detects_javascript_eval_as_security_finding` |
| Security lint policy | epic | - | implemented | verified | smoke | `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist` |
| CLI module registration | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard-cli registered_in_slice` |
| Security baseline accept/gate | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard-cli accept_snapshots_findings_then_scan_is_clean` |

## Security EC Profile

| Field | Value |
|---|---|
| ID | security-ec-profile |
| Root WI | - |
| Status | verified |
| Promise | AW EC and health treat guard output as first-class security evidence. |
| Required Verification | smoke |
| Gate Inventory | `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| AW health security metric | epic | - | implemented | verified | smoke | `./target/debug/aw ec check --project guard` |
| EC security evidence command | epic | - | implemented | verified | smoke | `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist` |

## Build & test

```bash
cargo test -p guard -p guard-cli
target/debug/guard scan projects/guard --profile security-lint --compact --no-persist
projects/guard/build.sh debug
```

## Non-goals

- AST ownership. `compass` remains the code-intelligence library.
- Environment isolation. `vat` owns runner/environment boundaries.
- E2E journey orchestration. `rig` owns executable behavior scenarios.
- Profiling/resource measurement. `meter` owns runtime/resource attribution.
- Benchmark comparison. `arena` owns N-target comparison and budgets.
- Upward evidence composition. guard is first-line: it scans source and
  consumes compass; it never drives vat/rig/meter/arena.
