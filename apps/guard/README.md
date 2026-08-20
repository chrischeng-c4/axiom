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
Tech design root: `apps/guard/tech-design`
Source ownership: TD-first source snapshots
Test gate: `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard`
CLI smoke: `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-ec-security-evidence-command`
Explicit non-goals: AST ownership, env isolation, e2e orchestration, profiling, benchmark comparison


## Capabilities

A promise with no gate under it is not claimed.

Nothing reads the tables below. The capability gate that validated their
shape was deleted with the `aw` binary, so the shape is convention now and
the commands named in each row are the only part that runs.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Static Security Scan | - | compass-backed security diagnostics normalized into `guard.report/1` |
| Security Policy Profile | - | `guard-baseline-static/1`, `guard-security-lint/1`, and `guard-strict/1` map security diagnostics/lint into policy findings |
| Lifecycle Security Report | #2930 | `guard.report/1` is a fail-closed, machine-readable security metric for lifecycle consumers |
| Dynamic Security Evidence | - | vat/rig/meter evidence adapters run and fold into `guard.report/1`; arena is legacy optional |

### Static Security Scan

guard scans source/config files with compass and emits a deterministic
`guard.report/1` security findings envelope.

- Root WI: none; this capability predates the tracker.
- Surfaces: CLI: `guard scan [path]` + `guard report` + `guard spec` +
  `guard llm` - Security scan, report reprojection, offline spec, and agent
  playbook entrypoints.
- Gate — behavior:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-scan-command-report-projection`
  - public scan projection
- Gate — security:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-compass-backed-diagnostic-scan`
  - compass-backed findings
- Gate — stability:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-stable-static-finding-normalization`
  - deterministic normalization
- Gate:
  `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard`
- Gate:
  `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo run -p guard --bin guard -- scan apps/guard --compact`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Compass-backed diagnostic scan | epic | - | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard --lib` |
| JSON report envelope | epic | - | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo run -p guard --bin guard -- scan apps/guard --compact` |
| Scan command report projection | change | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-scan-command-report-projection` |
| Stable static finding normalization | change | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-stable-static-finding-normalization` |

### Security Policy Profile

guard maps compass security diagnostics and security-impacting lint into policy
severities, remediation, locations, and agent prompts.

- Root WI: none; this capability predates the tracker.
- Surfaces: CLI: `guard scan --profile baseline-static` +
  `guard scan --profile security-lint` + `guard scan --profile strict` - Policy
  profile selection for baseline static, security lint, and strict security
  severity normalization.
- Gate — behavior:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-standalone-cli-distribution`
  - independently built public CLI surface
- Gate — security:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-security-lint-policy`
  - policy findings
- Gate — stability:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-stable-policy-selection`
  - repeatable policy selection
- Gate:
  `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard`
- Gate:
  `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo build -p guard --bin guard`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Baseline static policy | epic | - | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard --lib` |
| Security lint policy | epic | - | `CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard -- --nocapture` |
| Standalone CLI distribution | epic | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-standalone-cli-distribution` |
| Stable policy selection | change | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-stable-policy-selection` |

### Lifecycle Security Report

Guard emits a fail-closed `guard.report/1` decision that lifecycle consumers
can ingest without importing Guard internals. Consumer integration is owned and
verified by the consuming product.

- Root WI: none; this capability predates the tracker.
- Surfaces: CLI: `guard scan --profile security-lint --compact --no-persist` -
  lifecycle-consumable security evidence.
- Gate — behavior:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-security-report-consumer-contract`
  - consumer decision surface
- Gate — security:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-ec-security-evidence-command`
  - fail-closed evidence
- Gate — stability:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-stable-security-metric-projection`
  - stable health projection
- Gate:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-ec-security-evidence-command`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Lifecycle security metric | change | #2931 | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-lifecycle-security-metric` |
| EC security evidence command | epic | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-ec-security-evidence-command` |
| Security report consumer contract | change | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-security-report-consumer-contract` |
| Stable security metric projection | change | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-stable-security-metric-projection` |

### Dynamic Security Evidence

guard will compose static findings with vat-isolated execution, rig attack
journeys, and meter resource evidence. Legacy arena evidence can still be
passed through compatibility flags, but it is not required for production
readiness.

- Root WI: none; this capability predates the tracker.
- Surfaces: CLI:
  `guard scan --vat-runner <id> --vat-command <cmd> --rig-dir <path> --rig-scenario <path> --rig-command <cmd> --meter-target <path> --meter-command <cmd> --arena-spec <path> --arena-command <cmd>`
  - Dynamic security evidence adapters for isolated execution, exploit/e2e
  journeys, resource-abuse signals, and the legacy Arena compatibility surface.
- Gate — behavior:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-dynamic-adapter-routing`
  - exact routing and evidence folding for all nine public adapter flags
- Gate — security:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-ec-security-evidence-command`
  - fail-closed composed evidence
- Gate — stability:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-stable-evidence-folding`
  - repeatable folded evidence
- Gate:
  `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-ec-security-evidence-command`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Vat isolated security runner | epic | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-vat-isolated-security-runner` |
| Rig exploit journey bridge | epic | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-rig-exploit-journey-bridge` |
| Meter DoS/resource evidence bridge | epic | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-meter-dos-resource-evidence-bridge` |
| Dynamic adapter routing | change | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-dynamic-adapter-routing` |
| Stable evidence folding | change | - | `uv run --frozen --offline --project apps/guard/external-contracts python apps/guard/external-contracts/src/runner.py --case guard-stable-evidence-folding` |

## Build & test

```bash
cargo test -p guard
target/debug/guard scan apps/guard --profile security-lint --compact --no-persist
apps/guard/build.sh debug
```

## Non-goals

- AST ownership. `compass` remains the code-intelligence library.
- Environment isolation. `vat` owns runner/environment boundaries.
- E2E journey orchestration. `rig` owns executable behavior scenarios.
- Profiling/resource measurement. `meter` owns runtime/resource attribution.
- Benchmark comparison. `arena` owns N-target comparison and budgets.

## Brief

Security posture gate for the cclab ecosystem.

`guard` owns security policy and gate semantics. It does not replace
`compass`; it consumes `compass` as the static code-intelligence engine and
turns findings into one agent-readable report (`guard.report/1`). Dynamic
security evidence composes through the existing execution tools: `vat` for
isolated runs, `rig` for exploit/e2e journeys, `meter` for resource-abuse
evidence. Legacy `arena` flags remain accepted for compatibility, but arena is
no longer a required production evidence adapter.

## Contributing

Project-local authoring and verification rules live in [CONTRIBUTING.md](CONTRIBUTING.md).

## Capabilities

None claimed yet. A capability belongs here with the verbatim gate command that
verifies it; a promise with no gate under it is not claimed.
