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
window and a bounded ready/shutdown lifecycle; the registered-folder slice adds
the three-column shell without starting an agent process. Native agent PTY, cwd
synchronization, and context renderers remain separate child work items under
[#2171](https://github.com/chrischeng-c4/axiom/issues/2171).

## Registered Launch Folders

The left navigation registers real local directories through the native folder
picker. Workbench persists only canonical folder identity and the selected id;
the compact navigation state stays transient. Selection exposes one canonical
path to the future agent-launch boundary but does not set terminal cwd or start
a process.

The shell keeps three explicit landmarks visible: launch folders, terminal
preparation, and read-only context. Empty, cancelled-picker, invalid-path, and
constrained-width states remain actionable. The retained viewport and
interaction evidence for this slice lives under
[`evidence/folder-shell/2192`](evidence/folder-shell/2192/).

## Verification

```bash
cargo test -p workbench --test desktop_launch_smoke -- --nocapture
cargo test -p workbench --test folder_shell_journey -- --nocapture
```
