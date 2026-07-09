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
---
id: tape-vat-meter-guard-ec-gates-observability-flow
entry: route
nodes:
  route:
    kind: start
    label: "apps/tape gains vat.toml, guard-tape-security.toml, meter-tape-performance.toml, external-contracts/security-hardening + competitor-performance/efficiency/meter-gate.md, observability/, compose.yaml"
  vat_setup:
    kind: process
    label: "vat workspace clones the repo (base=../.., workdir=apps/tape); setup builds meter-cli + guard-cli plus tape's perf_gate/tape_vs_nats_jetstream/cli_contract test binaries"
  meter_runner:
    kind: process
    label: "runner meter-perf: ../../target/debug/meter test -- -p tape --test tape_perf_gate --test tape_vs_nats_jetstream -- --nocapture"
  guard_runner:
    kind: process
    label: "runner guard-security: ../../target/debug/guard scan apps/tape --profile security-lint --compact --no-persist --meter-command <cargo test -p tape --test cli_contract auth-relevant cases>"
  ec_efficiency:
    kind: process
    label: "apps/tape/aw.toml tool_contracts.tape-meter-performance -> meter-tape-performance.toml (SPEC-MANAGED, source_contract=tape-meter-performance-gate) delegates to vat run meter-perf"
  ec_security:
    kind: process
    label: "apps/tape/aw.toml tool_contracts.tape-guard-security -> guard-tape-security.toml (SPEC-MANAGED, source_contract=tape-security-hardening-guard-scan) delegates to vat run guard-security"
  root_aw_toml:
    kind: process
    label: "root aw.toml [[projects]] tape block gains ec.efficiency/ec.security bindings mirroring apps/relay + apps/keep"
  observability:
    kind: process
    label: "apps/tape/observability/{prometheus.yml,otel-collector-config.yaml,grafana-datasources.yaml} scrape tape's existing /metrics h2c endpoint (WI #1325 src/metrics.rs); compose.yaml runs tape+otel-collector+prometheus+jaeger+grafana locally"
  readme:
    kind: terminal
    label: "apps/tape/README.md EC Gates Configured row drops pending: language for vat.toml/meter-tape-performance.toml/guard-tape-security.toml"
edges:
  - { from: route, to: vat_setup }
  - { from: vat_setup, to: meter_runner }
  - { from: vat_setup, to: guard_runner }
  - { from: meter_runner, to: ec_efficiency }
  - { from: guard_runner, to: ec_security }
  - { from: ec_efficiency, to: root_aw_toml }
  - { from: ec_security, to: root_aw_toml }
  - { from: root_aw_toml, to: observability }
  - { from: observability, to: readme }
---
flowchart TD
    route[apps/tape gains vat.toml, guard-tape-security.toml, meter-tape-performance.toml, external-contracts, observability/, compose.yaml] --> vat_setup[vat workspace clones repo; setup builds meter-cli + guard-cli + tape test binaries]
    vat_setup --> meter_runner[runner meter-perf: meter test -- -p tape --test tape_perf_gate --test tape_vs_nats_jetstream]
    vat_setup --> guard_runner[runner guard-security: guard scan apps/tape --profile security-lint --meter-command cli_contract auth cases]
    meter_runner --> ec_efficiency[meter-tape-performance.toml SPEC-MANAGED source_contract=tape-meter-performance-gate delegates to vat run meter-perf]
    guard_runner --> ec_security[guard-tape-security.toml SPEC-MANAGED source_contract=tape-security-hardening-guard-scan delegates to vat run guard-security]
    ec_efficiency --> root_aw_toml[root aw.toml projects tape block gains ec.efficiency/ec.security bindings]
    ec_security --> root_aw_toml
    root_aw_toml --> observability[observability/ prometheus+otel-collector+grafana config; compose.yaml local dev stack]
    observability --> readme[README.md EC Gates Configured row drops pending language]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-vat-meter-guard-ec-gates-observability-verification
requirements:
  guard_manifest:
    id: R2
    text: "apps/tape/guard-tape-security.toml is a valid SPEC-MANAGED guard-cli native manifest bound to a source_contract in external-contracts/security-hardening."
    kind: functional
    risk: low
    verify: aw ec gen --verify apps/tape resolves tape-guard-security tool_contract
  meter_manifest:
    id: R3
    text: "apps/tape/meter-tape-performance.toml is a valid SPEC-MANAGED meter-cli native manifest bound to a source_contract in external-contracts/competitor-performance/efficiency."
    kind: functional
    risk: low
    verify: aw ec gen --verify apps/tape resolves tape-meter-performance tool_contract
  no_regression:
    id: R5
    text: "This config-only slice does not change tape's Rust source, so existing tape tests stay green."
    kind: regression
    risk: low
    verify: cargo test -p tape
  observability_config:
    id: R6
    text: "apps/tape/observability/{prometheus.yml,otel-collector-config.yaml,grafana-datasources.yaml} and apps/tape/compose.yaml are valid YAML and scrape/reference tape's real /metrics endpoint and OTLP port."
    kind: functional
    risk: low
    verify: manual review: prometheus.yml scrape_configs target matches tape's exposed port; compose.yaml service depends_on graph is acyclic
  root_aw_toml_bindings:
    id: R4
    text: "root aw.toml's [[projects]] tape block carries ec.efficiency and ec.security bindings that point at the vat runners."
    kind: functional
    risk: low
    verify: aw health --project tape --verify-ec reports both gates configured
  vat_config:
    id: R1
    text: "apps/tape/vat.toml declares a workspace + setup steps + meter-perf and guard-security runners that parse and resolve."
    kind: functional
    risk: low
    verify: aw ec gen --verify (or aw health --project tape --verify-ec) parses apps/tape/vat.toml without error
---
flowchart TD
    r1[R1 vat config] --> aw_ec_gen_verify_or_aw_health_project_tape_verify_ec_parses_apps_tape_vat_toml_without_error[aw ec gen --verify (or aw health --project tape --verify-ec) parses apps/tape/vat.toml without error]
    r2[R2 guard manifest] --> aw_ec_gen_verify_apps_tape_resolves_tape_guard_security_tool_contract[aw ec gen --verify apps/tape resolves tape-guard-security tool_contract]
    r3[R3 meter manifest] --> aw_ec_gen_verify_apps_tape_resolves_tape_meter_performance_tool_contract[aw ec gen --verify apps/tape resolves tape-meter-performance tool_contract]
    r4[R4 root aw toml bindings] --> aw_health_project_tape_verify_ec_reports_both_gates_configured[aw health --project tape --verify-ec reports both gates configured]
    r5[R5 no regression] --> cargo_test_p_tape[cargo test -p tape]
    r6[R6 observability config] --> manual_review_prometheus_yml_scrape_configs_target_matches_tape_s_exposed_port_compose_yaml_service_depends_on_graph_is_acyclic[manual review: prometheus.yml scrape_configs target matches tape's exposed port; compose.yaml service depends_on graph is acyclic]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/vat.toml
    action: create
    section: logic
    impl_mode: hand-written
    description: "vat workspace for tape's EC dispatch (mirrors apps/relay/vat.toml): base=../.., workdir=apps/tape, setup builds meter-cli+guard-cli plus tape_perf_gate/tape_vs_nats_jetstream/cli_contract test binaries; runners meter-perf (delegates to cargo test -p tape --test tape_perf_gate --test tape_vs_nats_jetstream) and guard-security (guard scan apps/tape --profile security-lint --meter-command attaching auth-boundary smoke evidence)."
  - path: apps/tape/guard-tape-security.toml
    action: create
    section: logic
    impl_mode: hand-written
    description: "SPEC-MANAGED guard-cli native manifest (mirrors apps/relay/guard-relay-security.toml): project=tape, source_contract=tape-security-hardening-guard-scan, delegate_command='cd apps/tape && ../../target/debug/vat run guard-security'."
  - path: apps/tape/meter-tape-performance.toml
    action: create
    section: logic
    impl_mode: hand-written
    description: "SPEC-MANAGED meter-cli native manifest (mirrors apps/relay/meter-relay-performance.toml): project=tape, source_contract=tape-meter-performance-gate, delegate_command='cd apps/tape && ../../target/debug/vat run meter-perf'."
  - path: apps/tape/external-contracts/security-hardening/security/security-evidence.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "New EC markdown (e2e-test + tool-contract sections, mirrors apps/relay's security-evidence.md): gate id tape-security-hardening-guard-scan, category security, binds guard-tape-security.toml as the tool contract."
  - path: apps/tape/external-contracts/competitor-performance/efficiency/meter-gate.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "New EC markdown (e2e-test + tool-contract sections, mirrors apps/relay's perf-gate.md): gate id tape-meter-performance-gate, category efficiency, binds meter-tape-performance.toml as the tool contract; distinct from tape's existing direct-cargo competitor-performance EC cases already in apps/tape/aw.toml."
  - path: apps/tape/observability/prometheus.yml
    action: create
    section: logic
    impl_mode: hand-written
    description: "Prometheus scrape config targeting tape's /metrics endpoint (mirrors projects/lumen/observability/prometheus.yml), scraping the otel-collector's Prometheus exporter port."
  - path: apps/tape/observability/otel-collector-config.yaml
    action: create
    section: logic
    impl_mode: hand-written
    description: "OTel collector config receiving OTLP gRPC from tape and re-exporting a Prometheus scrape target plus a trace exporter to Jaeger (mirrors projects/lumen/observability/otel-collector-config.yaml)."
  - path: apps/tape/observability/grafana-datasources.yaml
    action: create
    section: logic
    impl_mode: hand-written
    description: "Grafana datasource provisioning wiring Prometheus + Jaeger (mirrors projects/lumen/observability/grafana-datasources.yaml)."
  - path: apps/tape/compose.yaml
    action: create
    section: logic
    impl_mode: hand-written
    description: "Local dev compose stack: tape + otel-collector + prometheus + jaeger + grafana (mirrors projects/lumen/compose.yaml), building tape's Dockerfile and wiring the observability/ config volumes."
  - path: apps/tape/aw.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add aw.ec.generated.tool_manifests entries for tape-guard-security (guard-tape-security.toml) and tape-meter-performance (meter-tape-performance.toml), mirroring apps/relay/aw.toml's tool_manifests block."
  - path: aw.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Root aw.toml's [[projects]] tape block gains ec.efficiency = { tool = \"meter\", meter = \"apps/tape\", command = \"cd apps/tape && ../../target/debug/vat run meter-perf\" } and ec.security = { tool = \"guard\", dir = \"apps/tape\", command = \"cd apps/tape && ../../target/debug/vat run guard-security\" }, mirroring the relay/keep [[projects]] blocks."
  - path: apps/tape/README.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Update the 'EC Gates Configured' capability row/section to drop the pending: language for vat.toml, meter-tape-performance.toml (was meter-tape-replay*.toml in the WI title), and guard-tape-security.toml now that they exist, without touching unrelated rows."
```
