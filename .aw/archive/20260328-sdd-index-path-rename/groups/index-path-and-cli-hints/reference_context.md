---
change: sdd-index-path-rename
group: index-path-and-cli-hints
date: 2026-03-26
written_by: artifact_cli
review_verdict: approved
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| crates/cclab-sdd/logic/lens-index-storage.md | storage | high | Documents storage path resolution functions (resolve_lens_storage, resolve_pid_file, resolve_socket_path, resolve_cache_dir, resolve_module_index) in storage.rs, Current path pattern is {project_root}/.cclab/lens/ — must be renamed to {project_root}/cclab/.index/, All test assertions check for .cclab/lens suffix — must be updated to cclab/.index |
| crates/cclab-sdd/logic/implement-task.md | implementation-prompt | high | Documents build_implement_code_prompt() in create_change_impl.rs — the target function for CLI hints, Prompt templates section defines the CLI Commands block already present in prompt, New requirement: add code intelligence CLI hints (symbols, hover, references, impact, context) to this block, Hints should only be added when executor is mainthread (Claude has bash access) |
| crates/cclab-sdd/interfaces/cli/commands.md | cli-commands | medium | Documents the full cclab sdd CLI surface including daemon subcommands, Daemon subcommands include: symbols, hover, references, impact — the commands to hint, Context subcommand: cclab sdd context <targets...> [--depth N] |
| crates/cclab-sdd/logic/executor-resolution.md | executor | medium | Defines mainthread executor concept: executor == [mainthread] means prompt executes inline in Claude, Mainthread mode means Claude has Bash access and can run cclab CLI commands, subagent:* executors also have bash access when invoked by mainthread, Other executors (gemini, codex, claude-agent) spawn subprocesses without bash access |
| crates/cclab-sdd/logic/agent-context-builder.md | cli-commands | low | Background spec for cclab sdd context command (one of the CLI hints to add), Describes file:symbol target format and --depth parameter |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| index-path-rename | modify | crates/cclab-sdd/logic/lens-index-storage.md | overview, changes |
| cli-hints-impl-prompt | modify | crates/cclab-sdd/logic/implement-task.md | overview, changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: sdd-index-path-rename

**Verdict**: approved

### Summary

Reference context correctly identifies lens-index-storage.md and implement-task.md as primary specs. Spec plan covers both logical units with appropriate sections.

### Issues

No issues found.
