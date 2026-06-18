# guard

## Brief

Security posture gate for the cclab ecosystem.

`guard` owns security policy and gate semantics. It does not replace
`compass`; it consumes `compass` as the static code-intelligence engine and
turns findings into one agent-readable report (`guard.report/1`). Dynamic
security evidence composes through the existing execution tools: `vat` for
isolated runs, `rig` for exploit/e2e journeys, `meter` for resource-abuse
evidence. Legacy `arena` flags remain accepted for compatibility, but arena is
no longer a required production evidence adapter.

## Division of labor

```text
compass — AST, symbols, search, PDG/data-flow, generic security diagnostics
guard   — security policy/profile, findings, gate status, AW/EC integration
vat     — isolated local runner for risky checks
rig     — dynamic attack/e2e journeys
meter   — resource and DoS evidence
arena   — legacy optional compatibility for older budget/benchmark evidence
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
| `--arena-command <cmd>` / `--arena-spec <path>` | Run legacy optional arena security budget evidence |

Guard only owns security/policy lint. General formatting, style, and
non-security lint remain outside guard.

## AW Verification Snapshot

Last verified: 2026-06-13
Production readiness: ready for static security, security lint, and configured dynamic evidence
Tech design root: `projects/guard/tech-design`
Source ownership: TD-first source snapshots
Test gate: `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard -p guard-cli`
CLI smoke: `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard`
Health gate: `aw health --project guard`
Explicit non-goals: AST ownership, env isolation, e2e orchestration, profiling, benchmark comparison


## Capabilities

Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Static Security Scan | - | implemented | verified | smoke | ready | compass-backed security diagnostics normalized into `guard.report/1` |
| Security Policy Profile | - | implemented | verified | smoke | ready | `guard-baseline-static/1`, `guard-security-lint/1`, and `guard-strict/1` map security diagnostics/lint into policy findings |
| Security EC Profile | - | implemented | verified | smoke | ready | AW EC/health consumes guard reports as first-class security evidence |
| Dynamic Security Evidence | - | implemented | verified | smoke | ready | vat/rig/meter evidence adapters run and fold into `guard.report/1`; arena is legacy optional |

### Static Security Scan

ID: static-security-scan
Type: SecurityTool
Surfaces: CLI: `guard scan [path]` + `guard report` + `guard spec` + `guard llm` - Security scan, report reprojection, offline spec, and agent playbook entrypoints.
EC Dimensions: security: `guard` - compass-backed static source/config diagnostics normalized into guard.report/1 findings
Root WI: -
Status: verified
Required Verification: smoke
Promise:
guard scans source/config files with compass and emits a deterministic `guard.report/1` security findings envelope.
Gate Inventory:
- `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard`; `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo run -p guard-cli --bin guard -- scan projects/guard --compact`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Compass-backed diagnostic scan | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard scan::tests::detects_javascript_eval_as_security_finding` |
| JSON report envelope | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo run -p guard-cli --bin guard -- scan projects/guard --compact` |

### Security Policy Profile

ID: security-policy-profile
Type: SecurityTool
Surfaces: CLI: `guard scan --profile baseline-static` + `guard scan --profile security-lint` + `guard scan --profile strict` - Policy profile selection for baseline static, security lint, and strict security severity normalization.
EC Dimensions: security: `guard` - policy severity, remediation, and gate semantics for security findings
Root WI: -
Status: verified
Required Verification: smoke
Promise:
guard maps compass security diagnostics and security-impacting lint into policy severities, remediation, locations, and agent prompts.
Gate Inventory:
- `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard`; `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard-cli`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Baseline static policy | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard detects_javascript_eval_as_security_finding` |
| Security lint policy | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard -- --nocapture` |
| CLI module registration | epic | - | implemented | verified | smoke | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard-cli registered_in_slice` |

### Security EC Profile

ID: security-ec-profile
Type: SecurityTool
Surfaces: CLI: `aw ec check --project guard` + `guard scan --profile security-lint --compact --no-persist` - AW EC/health bridge and security-lint scan used as security evidence.
EC Dimensions: security: `aw ec + guard` - AW EC treats guard reports as first-class security gate evidence
Root WI: -
Status: verified
Required Verification: smoke
Promise:
AW EC and health treat guard output as first-class security evidence.
Gate Inventory:
- `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| AW health security metric | epic | - | implemented | verified | smoke | `./target/debug/aw ec check --project guard` |
| EC security evidence command | epic | - | implemented | verified | smoke | `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard` |

### Dynamic Security Evidence

ID: dynamic-security-evidence
Type: SecurityTool
Surfaces: CLI: `guard scan --vat-runner <id>` + `guard scan --rig-scenario <path>` + `guard scan --meter-target <path>` + `guard scan --meter-command <cmd>` - Dynamic security evidence adapters for isolated execution, exploit/e2e journeys, and resource-abuse signals.
EC Dimensions: security: `guard + vat + rig + meter` - dynamic security evidence folded into guard.report/1 from isolated runs, exploit journeys, and resource signals
Root WI: -
Status: verified
Required Verification: smoke
Promise:
guard will compose static findings with vat-isolated execution, rig attack journeys, and meter resource evidence. Legacy arena evidence can still be passed through compatibility flags, but it is not required for production readiness.
Gate Inventory:
- `target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Vat isolated security runner | epic | - | implemented | verified | smoke | `target/debug/guard scan projects/guard --compact --no-persist --vat-runner guard-security-smoke` |
| Rig exploit journey bridge | epic | - | implemented | verified | smoke | `target/debug/rig run --scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --compact` |
| Meter DoS/resource evidence bridge | epic | - | implemented | verified | smoke | `target/debug/guard scan projects/guard --compact --no-persist --meter-target projects/guard` |


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
