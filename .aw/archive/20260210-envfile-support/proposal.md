---
id: envfile-support
type: proposal
version: 1
created_at: 2026-02-10T02:29:52.928964+00:00
updated_at: 2026-02-10T02:29:52.928964+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add multi-level .env file support with variable substitution for Genesis agents."
history:
  - timestamp: 2026-02-10T02:29:52.928964+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 5
  new_files: 1
affected_specs:
  - id: envfile-support-spec
    path: specs/envfile-support-spec.md
    depends: []
---

<proposal>

# Change: envfile-support

## Summary

Add multi-level .env file support with variable substitution for Genesis agents.

## Why

Currently, Genesis agents rely on environment variables set in the host process, which is inflexible and makes managing multiple API keys or configurations difficult. \n\nThis change introduces support for loading environment variables from standard .env files. By supporting both a global envfile in the [workflow] section and provider-specific envfiles in [gemini], [codex], and [claude] sections, users gain fine-grained control over their environment. \n\nThe implementation will support industry-standard variable substitution, enabling more maintainable and DRY configuration files. This significantly improves the developer experience for setting up and running Genesis workflows across different environments.

## What Changes

- Add 'envfile' field to [workflow], [gemini], [codex], and [claude] sections in config.toml.
- Implement .env file loading with variable substitution support in cclab-genesis.
- Modify 'genesis_agent' tool to load and apply environment variables before executing agents.
- Ensure per-provider envfiles override global ones.

## Impact

- **Scope**: minor
- **Affected Files**: ~5
- **New Files**: ~1
- Affected specs:
  - `envfile-support-spec` (no dependencies)
- Affected code: `crates/cclab-genesis/src/models/change.rs`, `crates/cclab-genesis/src/mcp/tools/agent.rs`, `crates/cclab-genesis/src/orchestrator/script_runner.rs`

</proposal>
