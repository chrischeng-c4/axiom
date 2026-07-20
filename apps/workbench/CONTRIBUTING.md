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

Later slices add their own named integration target. The production journey
must retain viewport, accessibility, source-navigation, cwd, and recovery
evidence under its versioned evidence path.
