# wasm config rejects dev/build sections used by Cue

> **Issue**: #1403
> **Crate**: `jet` (`projects/jet/src/wasm_build/config.rs`)
> **Type**: bug

## Problem

`projects/jet/src/wasm_build/config.rs` declares the strict parse
struct:

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    wasm: Option<WasmConfig>,
}
```

…and `TOP_LEVEL_KEYS = &["wasm"]`. The non-WASM task-runner config
in `projects/jet/src/task_runner/config.rs` already accepts a much
larger set (`pipeline`, `dev`, `alias`, `build`, `resolve`, `test`,
`wasm`). When a project authors a single `jet.toml` and runs
`jet build --wasm` / `jet dev --wasm`, the WASM loader rejects the
file because of the legitimate `[dev]` / `[alias]` / `[build]`
sections used by the non-WASM workflows.

Cue's 2026-05-08 dogfood (`projects/cue/fe/jet.toml`) had to
delete its `[dev]` / `[dev.proxy]` / `[alias]` / `[build]` blocks and
move the values to CLI flags (`-p 3212`, `-o ../be/static`) as a
workaround. The shape it wants to keep:

```toml
[wasm]
entry = "src/CueWasmApp.tsx"
root_component = "CueWasmApp"

[dev]
port = 3212

[dev.proxy]
"/api" = "http://localhost:3210"

[alias]
"@/" = "./src/"

[build]
out_dir = "../be/static"
```

## Scope

In:

- Extend `ConfigFile` so the WASM loader accepts the non-WASM
  top-level sections that `JetConfig` already knows about
  (`pipeline`, `dev`, `alias`, `build`, `resolve`, `test`) without
  re-declaring their schemas — the WASM build doesn't need to do
  anything with them, it just needs to *not reject* them.
- Keep `deny_unknown_fields` enforcement: a section the canonical
  schema doesn't recognise still surfaces as `UnknownKey` with a
  did-you-mean hint.
- Update `TOP_LEVEL_KEYS` so the did-you-mean suggester ranks
  against the full known-section list.
- Update the existing `parse_str_with_warnings_surfaces_dev_port_anchor`
  test: the `dev-port` → `dev.port` deprecation remap now lands
  cleanly (the `[dev]` section is accepted), so the test must
  assert success + warning, not `UnknownKey` on `dev`.
- Add regression tests covering the Cue config shape end-to-end.

Out:

- Validating the contents of the non-WASM sections inside the WASM
  loader — those schemas are owned by `task_runner::config::JetConfig`
  and stay there. The WASM loader stores them as raw `toml::Value`
  so future code (e.g. `jet config lint --wasm`) can introspect.
- Proxy runtime, alias resolution, or build pipeline behaviour
  changes — this is purely a config-acceptance fix.

## Interface

`ConfigFile` grows to accept the shared sections as raw
`toml::Value`. `WasmConfig::parse_str` / `load_typed` keep returning
`WasmConfig` — the shared sections are simply not rejected.

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    wasm: Option<WasmConfig>,
    // Shared with non-WASM workflows; parsed as raw TOML so the
    // WASM build neither re-validates nor depends on the
    // JetConfig schema. Anchor: #1403.
    #[serde(default)] pipeline: Option<toml::Value>,
    #[serde(default)] dev:      Option<toml::Value>,
    #[serde(default)] alias:    Option<toml::Value>,
    #[serde(default)] build:    Option<toml::Value>,
    #[serde(default)] resolve:  Option<toml::Value>,
    #[serde(default)] test:     Option<toml::Value>,
}

const TOP_LEVEL_KEYS: &[&str] = &[
    "wasm", "pipeline", "dev", "alias", "build", "resolve", "test",
];
```

## Acceptance Criteria

- [x] `WasmConfig::parse_str` accepts a config containing `[wasm]`
      + `[dev]` + `[dev.proxy]` + `[alias]` + `[build]` and returns
      the parsed `WasmConfig`.
- [x] An *actually* unknown top-level key (`[bogus]`) still
      surfaces as `ConfigError::UnknownKey`.
- [x] Did-you-mean suggester picks `dev` for a typo `dvev` now that
      `dev` is in `TOP_LEVEL_KEYS`.
- [x] `parse_str_with_warnings` for `dev-port = 5173` returns the
      deprecation warning AND a successful parse (no `UnknownKey`
      on `dev`).
- [x] `jet build --wasm` and `jet dev --wasm` (via the underlying
      `WasmConfig::load`) load Cue's `[wasm] + [dev] + [alias] +
      [build]` config without error.
- [x] `cargo test -p jet --lib wasm_build::config` passes (all
      existing 24 tests + new coverage).

## Reference Context

- `projects/jet/src/wasm_build/config.rs:138-143` — `ConfigFile`
  struct.
- `projects/jet/src/wasm_build/config.rs:38` — `TOP_LEVEL_KEYS`.
- `projects/jet/src/task_runner/config.rs:208-210` — canonical
  `JET_TOP_LEVEL_KEYS` set that this aligns with.
- `projects/jet/src/wasm_build/config.rs:767-786` — the existing
  `parse_str_with_warnings_surfaces_dev_port_anchor` test that
  flips from "unknown key on dev" to "warning + success".
