---
id: tape-vat-meter-guard-ec-gates-observability
summary: >
  EC gate infra + observability slice for apps/tape (WI #1330, epic #1324),
  mirroring apps/relay's WI #1210 shape (vat-isolated meter/guard EC
  dispatch) rather than projects/lumen's larger multi-runner vat.toml, since
  tape has no external broker peer service to spin up for its efficiency
  gate (the NATS JetStream comparison in `tape_vs_nats_jetstream.rs` starts
  its own local `nats-server` subprocess and does not need a vat-managed
  backing service). Adds `apps/tape/vat.toml` (a vat workspace that builds
  meter-cli/guard-cli and the tape perf/security test binaries, then runs
  two runners: `meter-perf` delegating to `cargo test -p tape --test
  tape_perf_gate --test tape_vs_nats_jetstream`, and `guard-security`
  delegating to `guard scan` over `apps/tape` with a `--meter-command`
  attaching bearer-auth-boundary smoke evidence from
  `tests/cli_contract.rs`/`tests/behavior_tape_claim_*` auth-relevant
  cases). Adds `apps/tape/guard-tape-security.toml` and
  `apps/tape/meter-tape-performance.toml` SPEC-MANAGED tool manifests (the
  guard-cli/meter-cli native config schema, mirroring
  `apps/relay/guard-relay-security.toml` and
  `apps/relay/meter-relay-performance.toml`), each carrying a
  `source_contract` id resolving to two new EC markdown files:
  `apps/tape/external-contracts/security-hardening/security/security-evidence.md`
  (`e2e-test` + `tool-contract` sections, category `security`, gate id
  `tape-security-hardening-guard-scan`) and
  `apps/tape/external-contracts/competitor-performance/efficiency/meter-gate.md`
  (`e2e-test` + `tool-contract` sections, category `efficiency`, gate id
  `tape-meter-performance-gate` -- distinct from the existing
  `tape-competitor-performance-claim-closure`/`-local-regression-and-calibration-ledger`/`-nats-jetstream-replay-win`
  EC cases already in `apps/tape/aw.toml`, which stay as direct `cargo test`
  dispatch; this new gate is the vat-isolated meter-owned wrapper around the
  same perf_gate + nats_jetstream binaries, matching relay's
  `relay-competitor-performance-meter-gate` pattern of a meter-dispatch gate
  layered on top of pre-existing direct-cargo EC cases). Adds
  `apps/tape/observability/` (`prometheus.yml` scraping tape's `/metrics`
  h2c endpoint already implemented by WI #1325's `apps/tape/src/metrics.rs`,
  `otel-collector-config.yaml` receiving OTLP and re-exporting a Prometheus
  scrape target, `grafana-datasources.yaml` wiring Prometheus + Jaeger
  datasources) and `apps/tape/compose.yaml` (a local dev stack: tape +
  otel-collector + prometheus + jaeger + grafana, mirroring
  projects/lumen/compose.yaml exactly since both are h2c services with the
  same OTLP metrics surface). Finally wires `apps/tape/aw.toml`'s two new
  `tool_manifests` entries and root `aw.toml`'s `[[projects]]` tape block
  with `ec.efficiency`/`ec.security` bindings (mirroring the relay/keep
  `[[projects]]` blocks at root `aw.toml` lines ~414/431), and updates
  `apps/tape/README.md`'s "EC Gates Configured" row to drop the
  `pending:` language for `vat.toml`/`meter-tape-performance.toml`/
  `guard-tape-security.toml` now that they exist. No tape Rust source
  changes; this is EC gate config + observability config only, verified by
  `cargo build -p tape`/`cargo test -p tape` staying green (unaffected) plus
  `aw ec gen --verify`/`aw health --project tape --verify-ec` sanity-checking
  the new gate bindings parse and bind correctly.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
(fill)
```
