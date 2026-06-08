---
change: envfile-support
date: 2026-02-10
---

# Clarifications

## Q1: Envfile Format
- **Question**: What envfile format should be supported?
- **Answer**: Standard dotenv format with variable substitution support (e.g., ${VAR} or $VAR referencing earlier-defined or system env vars)
- **Rationale**: Dotenv is the industry standard. Variable substitution enables composing values from other vars without duplication.

## Q2: Config Level
- **Question**: Where should the envfile path be configured?
- **Answer**: Two levels: (1) Global envfile under [workflow] section, (2) Per-provider envfile under [gemini], [codex], [claude] sections. mainthread does not support envfile since it doesn't spawn a CLI process. Per-provider env vars override global ones when both are specified.
- **Rationale**: Global level provides shared env vars across all providers. Per-provider level allows provider-specific overrides (e.g., different API keys per provider). mainthread runs in the host process so envfile doesn't apply.

## Q3: Git Workflow
- **Question**: Which git workflow to use?
- **Answer**: in_place — work on current branch (main)
- **Rationale**: User preference for direct changes on current branch.

