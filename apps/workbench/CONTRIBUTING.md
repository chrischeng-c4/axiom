# Workbench Contributing

<!-- aw:meta:project-contributing:start -->
## Brief

Project-local contribution contract for Workbench.

## Authoritative Inputs

- Product promises and work roots: [CAPABILITIES.md](CAPABILITIES.md)
- Project orientation: [README.md](README.md)

## Local Workflow

Follow repository-level agent guidance and keep project-specific rules here.

## Verification

List the narrow commands that prove changes to Workbench.
<!-- aw:meta:project-contributing:end -->

## Workbench Authoring Rules

Follow the repository AW lifecycle. Keep each child WI bounded to its declared
host, PTY, cwd, renderer, provenance, adapter, or evidence ownership area.

Do not introduce vendor session/history models, AW mutation APIs, inferred cwd
from terminal text, or provider-specific provenance into the core boundary.

## Workbench Verification

The desktop bootstrap gate is:

```bash
cargo test -p workbench --test desktop_launch_smoke -- --nocapture
```

The registered-folder and rendered shell gate is:

```bash
cargo test -p workbench --test folder_shell_journey -- --nocapture
```

That test launches Jet's real headless Chromium runtime, records desktop and
constrained-width screenshots, and therefore needs permission to start the
browser process in sandboxed agent hosts. It must not be replaced with a DOM
mock or static string-only assertion.

The native-agent runtime gate is:

```bash
cargo test -p workbench --test pty_agent_adapters -- --nocapture
```

It must use the real platform PTY and deterministic local shell fixture. Do not
replace it with a PTY mock or make installed Claude Code, Codex, or AGY binaries
mandatory in CI.

The active cwd telemetry gate is:

```bash
cargo test -p workbench --test pty_cwd_context -- --nocapture
```

Only explicit OSC 7 file-URI frames may update active cwd. Never infer paths
from prompts, `cd` text, ordinary terminal output, or renderer content, and
never mutate the registered folder registry as a side effect of cwd telemetry.

The provider-neutral context renderer gate is:

```bash
cargo test -p workbench --test generic_context_renderers -- --nocapture
```

Keep the registry independent from PTY, active-cwd, registered-folder, and AW
runtime state. Renderers are read-only, path-confined, output-bounded adapters;
failures must produce a navigable fallback or allow the next compatible
renderer to run. Git probes and rendering commands must keep optional locks
disabled, and Markdown must not pass raw HTML or unsafe link targets through.

The optional AW typed renderer gate is:

```bash
cargo test -p workbench --test aw_typed_renderer -- --nocapture
```

Keep TD, EC, capability, and WI fixtures separate and byte-identical across
open, navigation, refresh, and close. The adapter may read `aw.toml` only as a
local activation signal; it must not invoke AW/GitHub, perform approvals or
lifecycle transitions, write repository state, or prevent generic Markdown
fallback when activation or typed structure is absent.

The provider-neutral provenance gate is:

```bash
cargo test -p workbench --test context_provenance -- --nocapture
```

Provider adapters must preserve repository-relative paths, one-based spans,
provider identity, and extracted/inferred/ambiguous classification. Only a
canonical regular file below the selected root may produce navigation; missing
or invalid inputs stay visible and non-authoritative. The provenance core must
not execute providers, AW, GitHub, verification commands, or repository writes.

The release-grade assembled journey gate is:

```bash
cargo test -p workbench --test production_journey -- --nocapture
```

This gate must use the real platform PTY with a deterministic local shell and
the same production session type used by native agents. It also runs Jet to
retain 1440x900 and 860x720 screenshots, keyboard/focus and accessibility
proof, unavailable-agent recovery, source navigation, transcript, context
summary, and a complete v1 manifest. UI controls require visible focus,
minimum 44px targets, 16px readable body text, no horizontal clipping, no
placeholder-only primary state, and reduced-motion behavior. Capability and EC
must keep the exact command above; do not replace it with mocks or temporary
evidence.

The production boundary leg must use the shared `configure_builder` Tauri IPC
handler and a real platform PTY; only the deterministic agent executable may be
substituted. It must retain at least twelve lifecycle cycles spanning
interrupt, terminate, and normal exit, prove that observed child pids are
reaped, keep every transcript at or below 524288 bytes, and enforce launch to
OSC7-ready at no more than 2000 ms and peak RSS at no more than 524288 KiB.
The measured values belong in the versioned `ipc-journey.json` evidence.

Later slices add their own named integration target. The production journey
must retain viewport, accessibility, source-navigation, cwd, and recovery
evidence under its versioned evidence path.
