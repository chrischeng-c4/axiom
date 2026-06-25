---
id: vat-network-sandbox-v3-seatbelt-egress-policy-deny-outbound-exce
summary: Give vat's macOS seatbelt backend a network egress policy (open | localhost-only | deny) so a `--isolation seatbelt` run can be made hermetic — denying outbound network except localhost via the sandbox-exec profile (codex's No-VM model), composing with v1/v2 routing so cooperating clients reach local emulators while external egress fails closed.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "vat can route traffic but cannot DENY egress. Extending the existing seatbelt profile with a network egress policy (Apple Seatbelt deny-by-default, like codex) confines a run's outbound network on macOS with no VM — the foundation of a hermetic sandbox that composes with v1/v2 routing."
---

# vat network sandbox v3: seatbelt egress policy (deny outbound except localhost)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-network-sandbox-v3-seatbelt-egress-logic
entry: start
nodes:
  start: { kind: start, label: "vat run with isolation seatbelt and network egress policy" }
  iso: { kind: decision, label: "isolation is seatbelt" }
  warn: { kind: process, label: "isolation none plus non-open policy warn once egress needs seatbelt" }
  pol: { kind: decision, label: "egress policy" }
  open: { kind: process, label: "open profile unchanged allow default" }
  local: { kind: process, label: "localhost-only deny network-outbound then allow remote localhost plus unix" }
  deny: { kind: process, label: "deny deny all network-outbound" }
  build: { kind: process, label: "profile_for emits write-confinement plus network lines" }
  run: { kind: process, label: "sandbox-exec -p profile -- runner" }
  effect: { kind: terminal, label: "runner reaches only localhost emulators external egress denied fail closed" }
edges:
  - { from: start, to: iso }
  - { from: iso, to: warn, label: "no" }
  - { from: warn, to: run }
  - { from: iso, to: pol, label: "yes" }
  - { from: pol, to: open, label: "open" }
  - { from: pol, to: local, label: "localhost-only" }
  - { from: pol, to: deny, label: "deny" }
  - { from: open, to: build }
  - { from: local, to: build }
  - { from: deny, to: build }
  - { from: build, to: run }
  - { from: run, to: effect }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-seatbelt-egress.schema.json"
title: "Seatbelt egress policy"
type: object
properties:
  egress:
    type: string
    enum: [open, localhost-only, deny]
    default: open
    description: "open = no network restriction (current behaviour). localhost-only = (deny network-outbound) + allow remote localhost/loopback + unix sockets. deny = block all outbound network."
  enforced_under:
    type: string
    const: seatbelt
    description: "Only enforceable via the seatbelt backend (sandbox-exec). isolation=none warns and runs open."
  reads:
    type: string
    const: unaffected
    description: "Reads stay open (toolchains resolve); only outbound network is filtered."
additionalProperties: true
```
## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-network-egress.schema.json"
title: "vat.toml [network].egress"
type: object
properties:
  network:
    type: object
    properties:
      egress:
        type: string
        enum: [open, localhost-only, deny]
        description: "Default open. Selects the seatbelt egress policy for the run."
      routes:
        type: array
        description: "Existing v1 routes (unchanged); coexist with egress under [network]."
        items: { type: object }
    additionalProperties: true
  note:
    type: string
    description: "localhost-only is the hermetic-with-routing mode: vat's emulators/proxy bind 127.0.0.1 so they stay reachable while external hosts are denied."
additionalProperties: true
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat run
    behavior:
      - "Reads [network].egress (default open). Under --isolation seatbelt, the egress policy is baked into the sandbox-exec profile: localhost-only denies outbound network except localhost (loopback + unix sockets); deny blocks all outbound; open is unchanged."
      - "Under --isolation none, a non-open egress policy is not enforceable — vat runs the command and prints a one-line warning that egress confinement requires seatbelt."
      - "Reads remain unrestricted; the GPU/host-process model is untouched."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-network-sandbox-v3-seatbelt-egress-unit-tests
---
requirementDiagram
    requirement open_profile_unchanged {
      id: UT1
      text: "profile_for with egress=open is byte-identical to today's profile (no network lines added)."
      risk: high
      verifymethod: test
    }
    requirement localhost_only_profile {
      id: UT2
      text: "profile_for with egress=localhost-only contains (deny network-outbound) and a localhost allow; deny contains a full network-outbound deny with no localhost allow."
      risk: medium
      verifymethod: test
    }
    test seatbelt_egress_open_unchanged_tests {
      type: functional
      verifies: open_profile_unchanged
    }
    test seatbelt_egress_profile_tests {
      type: functional
      verifies: localhost_only_profile
    }
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-seatbelt-egress-smoke
    name: "seatbelt localhost-only denies external egress, allows localhost"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_sandbox_egress -- --nocapture"
    assertions:
      - "under sandbox-exec with the localhost-only profile, a process connecting to a localhost listener succeeds while a connect to a non-loopback address is denied. Skips cleanly if sandbox-exec is absent or not macOS."
  - id: vat-seatbelt-egress-build
    name: "default + lean build compile"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo build -p vat --no-default-features"
    assertions:
      - "vat compiles with and without default features; egress policy is independent of the emulator feature."
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/sandbox/seatbelt.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "profile_for takes an egress policy and appends the network lines: localhost-only → (deny network-outbound) + (allow network* (remote ip \"localhost:*\")) + unix sockets; deny → full outbound deny; open → unchanged. Thread the policy through resolve()/Sandbox."
  - path: projects/vat/src/config.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add the [network].egress enum (open | localhost-only | deny, default open) on NetworkConfig (alongside the v1 routes field)."
  - path: projects/vat/src/sandbox/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Plumb the egress policy into the Sandbox trait / seatbelt resolve call site."
  - path: projects/vat/src/commands/run.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Pass [network].egress to the seatbelt backend; warn once when a non-open policy is set with isolation != seatbelt."
  - path: projects/vat/tests/vat_sandbox_egress.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "Profile-string assertions + a sandbox-exec localhost-only smoke test that skips when unavailable."
```
