---
id: agent-skill
type: spec
title: "/cclab:sdd:agent Skill"
version: 1
spec_type: algorithm
spec_group: cclab-sdd
created_at: 2026-02-23T00:00:00+00:00
updated_at: 2026-02-23T00:00:00+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# /cclab:sdd:agent Skill

## Overview
<!-- type: overview lang: markdown -->

A Claude Code skill that provides unified agent invocation via `sdd_delegate_agent`. It accepts a provider:model agent identifier, an action type, and a prompt, then delegates to the appropriate LLM backend (Gemini, Codex, or Claude).

This skill is used for ad-hoc exploration and review only. Workflow agent delegation is now handled internally by workflow tools via `run_agent()` — this skill is no longer part of the workflow loop.

**Template source**: `crates/cclab-sdd/templates/mainthread/skills/cclab-sdd-agent/SKILL.md`

**Cross-reference**: `tools/delegate-agent.md` for the `sdd_delegate_agent` CLI tool implementation.

## Template
<!-- type: doc lang: markdown -->

```markdown
---
name: cclab:sdd:agent
description: Run prompts with any LLM agent (Gemini, Codex, Claude) via unified tool
user-invocable: true
---

# /cclab:sdd:agent

Unified agent invocation via the `sdd_delegate_agent` CLI tool. Supports Gemini, Codex, and Claude with action-based prompt templates.

## Usage
<!-- type: doc lang: markdown -->

` ` `bash
/cclab:sdd:agent <agent> <action> "<prompt>"
` ` `

**Agent format**: `provider:model_id`
- `gemini:flash`, `gemini:pro`
- `codex:fast`, `codex:balanced`, `codex:deep`, `codex:max`
- `claude:fast`, `claude:balanced`, `claude:deep`

**Actions**: `explore`, `review`, `custom`

## Instructions
<!-- type: doc lang: markdown -->

1. Parse the user's arguments to extract agent, action, and prompt.
   - If no agent is specified, default to `gemini:flash`
   - If no action is specified, infer from context:
     - Words like "search", "find", "explore", "analyze" -> `explore`
     - Words like "review", "check", "audit", "critique" -> `review`
     - Otherwise -> `custom`

2. Invoke the agent CLI directly based on provider:

` ` `bash
# Gemini
gemini -m <model_id> "<prompt>"

# Codex
codex exec --model <model_id> --full-auto "<prompt>"

# Claude
claude --model <model_id> -p "<prompt>"
` ` `

3. Report the result to the user, including usage metrics if available.

## Examples
<!-- type: doc lang: markdown -->

` ` `bash
# Explore with Gemini Flash (default)
/cclab:sdd:agent gemini:flash explore "Find all files that handle authentication"

# Review with Codex
/cclab:sdd:agent codex:balanced review "Review src/auth.rs for security vulnerabilities"

# Custom prompt with Claude
/cclab:sdd:agent claude:fast custom "Explain the sdd workflow architecture"

# Short form - agent and action inferred
/cclab:sdd:agent "Search for all CLI tool definitions"
` ` `
```

## Installation
<!-- type: doc lang: markdown -->

### R1 - Compile-Time Embedding

```yaml
id: R1
priority: high
status: draft
```

The template is embedded via `include_str!()` as `SKILL_AGENT` in `init.rs:16`. The installer does NOT read from the filesystem at runtime.

### R2 - Installation Path

```yaml
id: R2
priority: high
status: draft
```

`cclab init` writes the template to `.claude/skills/cclab-sdd-agent/SKILL.md`. The parent `.claude/skills/` directory is created if it does not exist. The file is **overwritten on every `cclab init`** — this is a system-managed skill, not user-editable.

## Agent Format
<!-- type: doc lang: markdown -->

### R3 - Provider:Model Syntax

```yaml
id: R3
priority: high
status: draft
```

Agents are specified as `provider:model_id`. Available combinations:

| Provider | Model IDs | Example |
|----------|-----------|---------|
| `gemini` | `flash`, `pro` | `gemini:flash` |
| `codex` | `fast`, `balanced`, `deep`, `max` | `codex:balanced` |
| `claude` | `fast`, `balanced`, `deep` | `claude:deep` |

Default agent when none specified: `gemini:flash`.

Model IDs map to full model names via `.aw/config.toml` provider sections (see `config.md`).

## Action Inference
<!-- type: doc lang: markdown -->

### R4 - Keyword-Based Action Detection

```yaml
id: R4
priority: medium
status: draft
```

When the user omits the action parameter, the skill infers it from the prompt text:

| Keywords | Inferred Action |
|----------|----------------|
| search, find, explore, analyze | `explore` |
| review, check, audit, critique | `review` |
| *(anything else)* | `custom` |

Keyword matching is case-insensitive and checks individual words in the prompt.

## Test Plan
<!-- type: doc lang: markdown -->

| Test | Covers |
|------|--------|
| embedded_template_installed_by_init | R1, R2 |
| provider_model_agent_format_routes_to_backend | R3 |
| omitted_action_is_inferred_from_keywords | R4 |
