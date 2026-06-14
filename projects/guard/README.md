# guard

Security posture gate for the cclab ecosystem.

`guard` owns security policy and gate semantics. It does not replace
`compass`; it consumes `compass` as the static code-intelligence engine and
turns findings into one agent-readable report (`guard.report/1`). Dynamic
security evidence composes through the existing execution tools: `vat` for
isolated runs, `rig` for exploit/e2e journeys, `meter` for resource-abuse
evidence, and `arena` for budget/benchmark comparison.

## Division of labor

```text
compass — AST, symbols, search, PDG/data-flow, generic security diagnostics
guard   — security policy/profile, findings, gate status, AW/EC integration
vat     — isolated local runner for risky checks
rig     — dynamic attack/e2e journeys
meter   — resource and DoS evidence
arena   — comparative security-performance budgets
```

## Mental model

```text
guard scan .
  run compass security diagnostics across supported source/config languages
  normalize them into guard findings
  rank by policy severity
  persist .guard/last-report.json
  print ONE guard.report/1 JSON -> exit 0 clean / 1 findings / 3+ tool error
```

## Verbs

| Command | Effect |
|---------|--------|
| `guard scan [path]` | Run the baseline static security profile over a file or directory |
| `guard report` | Re-project `.guard/last-report.json` without scanning |
| `guard spec` | Offline self-description of the current report/policy surface |
| `guard llm` | Offline agent playbook |

## Scan profiles and evidence

| Flag | Effect |
|---|---|
| `--profile baseline-static` | Compass security diagnostics only |
| `--profile security-lint` | Security diagnostics plus security-impacting lint, including supply-chain Docker tags and SQL injection helpers |
| `--profile strict` | Security-lint profile with stricter severity normalization |
| `--vat-runner <id>` | Run a named `vat.toml` runner as isolated security evidence |
| `--rig-scenario <path>` / `--rig-dir <dir>` | Run exploit/e2e journey evidence through rig |
| `--meter-target <path>` / `--meter-command <cmd>` | Run resource/DoS evidence through meter |
| `--arena-command <cmd>` / `--arena-spec <path>` | Run arena security budget evidence |

Guard only owns security/policy lint. General formatting, style, and
non-security lint remain outside guard.

## Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| static-security-scan | - | implemented | verified | smoke | ready | compass-backed security diagnostics normalized into `guard.report/1` |
| security-policy-profile | - | implemented | verified | smoke | ready | `guard-baseline-static/1`, `guard-security-lint/1`, and `guard-strict/1` map security diagnostics/lint into policy findings |
| security-ec-profile | - | implemented | verified | smoke | ready | AW EC/health consumes guard reports as first-class security evidence |
| dynamic-security-evidence | - | implemented | verified | smoke | ready | vat/rig/meter/arena evidence adapters run and fold into `guard.report/1` |

## AW Verification Snapshot

| Field | Value |
|---|---|
| Last verified | 2026-06-13 |
| Production readiness | ready for static security, security lint, and configured dynamic evidence |
| Tech design root | `projects/guard/tech-design` |
| Source ownership | TD-first source snapshots |
| Test gate | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard -p guard-cli` |
| CLI smoke | `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard --arena-command "target/debug/arena spec --compact"` |
| Health gate | `aw health --project guard` |
| Explicit non-goals | AST ownership, env isolation, e2e orchestration, profiling, benchmark comparison |

## Static Security Scan

| Field | Value |
|---|---|
| ID | static-security-scan |
| Root WI | - |
| Status | verified |
| Promise | guard scans source/config files with compass and emits a deterministic `guard.report/1` security findings envelope. |
| Required Verification | smoke |
| Gate Inventory | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard`; `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo run -p guard-cli --bin guard -- scan projects/guard --compact` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Compass-backed diagnostic scan | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard scan::tests::detects_javascript_eval_as_security_finding` |
| JSON report envelope | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo run -p guard-cli --bin guard -- scan projects/guard --compact` |

## Security Policy Profile

| Field | Value |
|---|---|
| ID | security-policy-profile |
| Root WI | - |
| Status | verified |
| Promise | guard maps compass security diagnostics and security-impacting lint into policy severities, remediation, locations, and agent prompts. |
| Required Verification | smoke |
| Gate Inventory | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard`; `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard-cli` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Baseline static policy | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard detects_javascript_eval_as_security_finding` |
| Security lint policy | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard -- --nocapture` |
| CLI module registration | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard-cli registered_in_slice` |

## Security EC Profile

| Field | Value |
|---|---|
| ID | security-ec-profile |
| Root WI | - |
| Status | verified |
| Promise | AW EC and health treat guard output as first-class security evidence. |
| Required Verification | smoke |
| Gate Inventory | `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard --arena-command "target/debug/arena spec --compact"` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| AW health security metric | epic | - | implemented | verified | smoke | `./target/debug/aw ec check --project guard` |
| EC security evidence command | epic | - | implemented | verified | smoke | `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard --arena-command "target/debug/arena spec --compact"` |

## Dynamic Security Evidence

| Field | Value |
|---|---|
| ID | dynamic-security-evidence |
| Root WI | - |
| Status | verified |
| Promise | guard will compose static findings with vat-isolated execution, rig attack journeys, meter resource evidence, and arena security-performance budgets. |
| Required Verification | smoke |
| Gate Inventory | `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard --arena-command "target/debug/arena spec --compact"` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Vat isolated security runner | epic | - | implemented | verified | smoke | `target/debug/guard scan projects/guard --compact --no-persist --vat-runner guard-security-smoke` |
| Rig exploit journey bridge | epic | - | implemented | verified | smoke | `target/debug/rig run --scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --compact` |
| Meter DoS/resource evidence bridge | epic | - | implemented | verified | smoke | `target/debug/guard scan projects/guard --compact --no-persist --meter-target projects/guard` |
| Arena security budget bridge | epic | - | implemented | verified | smoke | `target/debug/guard scan projects/guard --compact --no-persist --arena-command "target/debug/arena spec --compact"` |

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
