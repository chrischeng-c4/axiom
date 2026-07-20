# Workbench

<!-- aw:meta:project-readme:start -->
## Brief

Describe the agent-readable purpose of Workbench.

## Contributing

Project-local authoring and verification rules live in [CONTRIBUTING.md](CONTRIBUTING.md).

## Capability Contract

Product promises and work roots live in [CAPABILITIES.md](CAPABILITIES.md).
<!-- aw:meta:project-readme:end -->

## Product Brief

Workbench is a terminal-first native desktop shell for Claude Code, Codex, and
AGY. It keeps each agent's real CLI and session model authoritative while
placing optional, read-only project context beside the terminal.

## Product Boundary

- Registered folders select the working directory used to launch a native
  agent process; they are not a second project or session database.
- A real PTY owns terminal input, output, resize, signals, exit, and current
  working directory.
- Context renderers are optional views over canonical repository sources.
- AW TD, EC, capability, and WI documents are one read-only renderer input.
  Workbench never advances or duplicates the AW lifecycle.
- Repositories without AW remain useful through Markdown and Git context.

## Desktop Stack

The native host is Rust plus Tauri 2. Its bootstrap proves one local WebView
window and a bounded ready/shutdown lifecycle. The assembled three-column shell
keeps registered-folder identity, native PTY sessions, active cwd telemetry,
and read-only context in separate modules joined by the production journey
under [#2171](https://github.com/chrischeng-c4/axiom/issues/2171).

## Registered Launch Folders

The left navigation registers real local directories through the native folder
picker. Workbench persists only canonical folder identity and the selected id;
the compact navigation state stays transient. Selection exposes one canonical
path to the native agent-launch boundary; registration itself never sets
terminal cwd or starts a process.

The shell keeps three explicit landmarks visible: launch folders, terminal
preparation, and read-only context. Empty, cancelled-picker, invalid-path, and
constrained-width states remain actionable. The retained viewport and
interaction evidence for this slice lives under
[`evidence/folder-shell/2192`](evidence/folder-shell/2192/).

## Native Agent PTY

Workbench constructs one inspectable native command for Claude Code, Codex, or
AGY; Claude Code is the initial default. The command keeps the selected launch
folder as child cwd and adds no hidden resume or history arguments. If the
selected binary is unavailable, launch returns a recoverable error before PTY
allocation so another agent can be selected immediately.

The runtime uses a real native pseudo-terminal for input, output, terminal
resize, Ctrl-C, child exit status, explicit termination, and abandoned-session
cleanup. It does not store vendor sessions or derive context cwd from terminal
text. The production journey composes this runtime with authoritative
cwd-to-context synchronization without adding vendor session ownership.

## Authoritative Active Cwd

Active context follows explicit OSC 7 `file://localhost/...` control frames
from the PTY stream. Workbench validates and canonicalizes each framed path,
accepts only an existing local directory, and discloses OSC 7 as the update
source. Ordinary prompt text, shell output, malformed or remote URIs, missing
paths, and files never become cwd.

Active cwd is ephemeral runtime context. It does not add, remove, rename, or
reselect registered launch folders; those remain user-owned launch identity.

## Generic Context Renderers

The provider-neutral renderer registry selects compatible renderers by
descending priority and stable renderer id. A failed renderer contributes a
visible warning and the registry continues to the next candidate; unsupported,
missing, corrupt, or oversized artifacts retain a navigable fallback instead
of breaking the terminal surface.

The Markdown renderer reads at most one MiB of UTF-8 source, escapes raw HTML,
and neutralizes unsafe link targets before emitting HTML. The Git renderer runs
only read-only status and diff commands with optional locks disabled, bounds
their output, and exposes changed paths for source navigation. Both operate on
ordinary repositories without `aw.toml`, remain confined to the selected root,
and have no dependency on PTY or cwd runtime state.

## Optional AW Typed Context

When the selected root contains an `aw.toml`, Workbench can opt into a
higher-priority, read-only renderer for structurally recognized TD, EC,
capability, and WI Markdown. It exposes frontmatter, typed sections, Mermaid,
commands, assertions, explicit artifact relationships, and source-line labels;
all values are escaped and the underlying Markdown remains the canonical
repository source.

The adapter never invokes `aw`, GitHub, approval, or lifecycle commands. Open,
navigate, refresh, and close are bounded reads with no retained mutable handle.
If configuration or typed structure is absent, the same file continues through
the generic Markdown renderer; parse failures are isolated by the registry.

## Canonical Context Provenance

Every provider can describe context with the same repository-relative file,
optional one-based span, provider identity, and extracted/inferred/ambiguous
classification. Extracted items become authoritative navigation only when the
file and span resolve beneath the selected canonical root. Missing files,
invalid spans, traversal, directories, and symlink escape remain visible as
non-authoritative states and never receive fabricated links.

Inferred and ambiguous context is always labeled derived and retains every
input location, including unavailable inputs. The model is serializable and
provider-neutral; resolving it performs metadata/canonicalization reads only.
Repository source and executable verification evidence remain canonical.

## Production Journey

The assembled desktop path now starts with a registered canonical folder,
launches the selected Claude Code, Codex, or AGY binary through the real native
PTY, streams bounded terminal output, accepts input/resize/interrupt/terminate,
and updates active cwd only from OSC 7. Missing agent binaries stay recoverable:
the folder and read-only context remain available while another agent is
selected.

The center pane exposes agent choice, active cwd and telemetry source, terminal
transcript, input, and lifecycle controls. The context pane switches between
Git, Markdown, and configured AW typed views with renderer identity,
provenance, warnings, and canonical source navigation. Controls use visible
focus and 44px targets; the constrained desktop layout retains readable,
placeholder-free primary state and respects reduced motion.

Release evidence is retained under
[`evidence/production-journey/v1`](evidence/production-journey/v1/). Its
manifest maps the real-PTY transcript, context summary, desktop and constrained
screenshots, accessibility assertions, recovery, and source-navigation proof
to the same Cargo command used by the capability and external contract.

## Verification

```bash
cargo test -p workbench --test desktop_launch_smoke -- --nocapture
cargo test -p workbench --test folder_shell_journey -- --nocapture
cargo test -p workbench --test pty_agent_adapters -- --nocapture
cargo test -p workbench --test pty_cwd_context -- --nocapture
cargo test -p workbench --test generic_context_renderers -- --nocapture
cargo test -p workbench --test aw_typed_renderer -- --nocapture
cargo test -p workbench --test context_provenance -- --nocapture
cargo test -p workbench --test production_journey -- --nocapture
```
