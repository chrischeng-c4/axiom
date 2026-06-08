# Requirements: score-init-bootstrap / default

## Issue References

- enhancement-score-init-command-bootstrap-claude-assets

## Requirements

### Wire init into CLI
- R1: Add `Init` variant to `Commands` enum in `projects/score/cli/src/commands.rs` with fields `name: Option<String>`, `force: bool`
- R2: Dispatch `Commands::Init` to `init::run()` in `run_command()`
- R3: `score init --help` documents the command purpose and flags

### Bootstrap agent definitions
- R4: Add all 5 `score-*` agent definitions to `projects/score/cli/templates/mainthread/agents/` as `include_str!` constants in `init.rs`
- R5: `init.rs::install_agents()` writes them to `.claude/agents/` in the target project
- R6: Remove legacy `sdd-*` agent files during init update

### Bootstrap hook scripts
- R7: Add hook scripts (`score-safe-bash.sh`, `score-readonly-bash.sh`, `score-next-step.sh`) to `projects/score/cli/templates/mainthread/hooks/`
- R8: `init.rs::install_hooks()` writes them to `.claude/hooks/` with executable permissions (`chmod +x`)

### Bootstrap settings.json
- R9: Add `settings.json` template to `projects/score/cli/templates/mainthread/` with SubagentStop hook registration matching `score-*` pattern
- R10: `init.rs` merges the template with any existing `.claude/settings.json` (preserve user customizations)
- R11: If the existing `settings.json` already has a SubagentStop hook, warn the user and skip

### Complete skill set
- R12: Add `score-issue` skill to templates and install in `init.rs::install_skills()`
- R13: Add `score-issue-patrol` skill to templates and install

### Update behavior
- R14: Running `score init` on an already-initialized project updates all assets while preserving `project.md` and user customizations
- R15: Version check prevents downgrades (existing behavior — keep)
