---
id: relay-cli-std-surface
summary: >
  Replace relay's bare relay-server binary with a single `relay` CLI bin
  (clap): bare `relay` serves the h2c transport (env-fallback flags), and the
  standard agent-facing ops ship alongside it — `relay llm` (cli_std::llm over
  relay-supplied topics + build-stamp ToolInfo), `relay upgrade`, and
  `relay issue <search|view|create>` (cli_std::issue, auto-tagged
  project:relay). build.rs delegates to libs/build-stamp for
  RELAY_GIT_SHA/RELAY_BUILT_AT/RELAY_TARGET. relay-raft stays untouched (its
  collapse is the raft-host adoption WI).
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-cli-std-surface-contract
entry: parse
nodes:
  parse: { kind: start, label: "relay CLI parses argv with clap (single relay bin)" }
  branch: { kind: decision, label: "which subcommand" }
  serve: { kind: process, label: "bare relay runs the h2c server with ServeArgs flags falling back to RELAY_BIND and RELAY_DATA_DIR env" }
  llm: { kind: process, label: "llm renders relay topics via cli_std llm render with the build-stamp ToolInfo" }
  upgrade: { kind: process, label: "upgrade checks or installs the latest relay release asset via cli_std (network path behind self-update feature)" }
  issue: { kind: process, label: "issue search view create dispatch cli_std issue filtered and auto-tagged project:relay (network path behind issue feature)" }
  out: { kind: terminal, label: "serve runs until shutdown; ops verbs print to stdout and exit" }
edges:
  - { from: parse, to: branch }
  - { from: branch, to: serve, label: "none (default)" }
  - { from: branch, to: llm, label: "llm" }
  - { from: branch, to: upgrade, label: "upgrade" }
  - { from: branch, to: issue, label: "issue" }
  - { from: serve, to: out }
  - { from: llm, to: out }
  - { from: upgrade, to: out }
  - { from: issue, to: out }
---
flowchart TD
    parse([relay CLI parses argv with clap single relay bin]) --> branch{which subcommand}
    branch -->|none default| serve[bare relay runs the h2c server with ServeArgs flags falling back to RELAY_BIND and RELAY_DATA_DIR env]
    branch -->|llm| llm[llm renders relay topics via cli_std llm render with the build-stamp ToolInfo]
    branch -->|upgrade| upgrade[upgrade checks or installs the latest relay release asset via cli_std network path behind self-update feature]
    branch -->|issue| issue[issue search view create dispatch cli_std issue filtered and auto-tagged project relay network path behind issue feature]
    serve --> out([serve runs until shutdown and ops verbs print to stdout and exit])
    llm --> out
    upgrade --> out
    issue --> out
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-cli-std-surface-verification
requirements:
  build_stamp_feeds_toolinfo:
    id: R3
    text: "RELAY_GIT_SHA, RELAY_BUILT_AT, and RELAY_TARGET are emitted by libs/build-stamp and populate the cli-std ToolInfo used by llm/upgrade/issue."
    kind: functional
    risk: low
    verify: toolinfo_is_stamped test in projects/relay/src/bin/relay.rs
  cli_parses_convention_verbs:
    id: R1
    text: "The relay CLI parses llm, upgrade, and issue search/view/create with their convention flags, and bare relay (no subcommand) parses as serve."
    kind: functional
    risk: medium
    verify: cli_parse_surface tests in projects/relay/src/bin/relay.rs
  serve_replaces_relay_server:
    id: R2
    text: "Bare relay serves exactly what relay-server served (RELAY_BIND/RELAY_DATA_DIR honored); the relay-server binary is removed from the crate."
    kind: regression
    risk: medium
    verify: projects/relay/tests/http2_transport.rs against the relay serve router
---
flowchart TD
    r1[R1 cli parses convention verbs] --> cli_parse_surface_tests_in_projects_relay_src_bin_relay_rs[cli_parse_surface tests in projects/relay/src/bin/relay.rs]
    r2[R2 serve replaces relay server] --> projects_relay_tests_http2_transport_rs_against_the_relay_serve_router[projects/relay/tests/http2_transport.rs against the relay serve router]
    r3[R3 build stamp feeds toolinfo] --> toolinfo_is_stamped_test_in_projects_relay_src_bin_relay_rs[toolinfo_is_stamped test in projects/relay/src/bin/relay.rs]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Replace [[bin]] relay-server with [[bin]] relay (src/bin/relay.rs); add clap + cli-std (default-features = false) deps and the build-stamp build-dependency; add self-update/issue features mapping to cli-std/online (keep's feature layout, with the report-issue alias omitted)."
  - path: projects/relay/src/bin/relay.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Single relay CLI bin (clap): bare relay (no subcommand) runs the h2c server with ServeArgs flags falling back to RELAY_BIND/RELAY_DATA_DIR env (the relay_server.rs behavior verbatim); Command::Llm/Upgrade/Issue dispatch to cli_std::{llm,upgrade,issue} with relay's ToolInfo; mirrors projects/keep/src/bin/keep.rs."
  - path: projects/relay/src/bin/relay_server.rs
    action: delete
    section: logic
    impl_mode: hand-written
    description: "Removed: bare relay serve replaces the relay-server entrypoint."
  - path: projects/relay/build.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Delegate to build_stamp::stamp(\"RELAY\") so RELAY_GIT_SHA/RELAY_BUILT_AT/RELAY_TARGET feed ToolInfo — no hand-rolled git/timestamp logic."
  - path: projects/relay/src/llm.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "relay's cli_std::llm::Topic list (outline, http-api, operations) + the stamped ToolInfo constructor shared by llm/upgrade/issue."
  - path: projects/relay/Dockerfile
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Build/copy the relay bin instead of relay-server (relay-raft line untouched)."
  - path: projects/relay/src/bin/relay.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "#[cfg(test)] mod: clap parse tests for llm/upgrade/issue verbs + bare-serve default (cli_parse_surface), and toolinfo_is_stamped asserting the build-stamp envs populate ToolInfo."
```
