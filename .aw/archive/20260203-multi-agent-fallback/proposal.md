---
change: multi-agent-fallback
title: "Multi-agent fallback per workflow stage"
description: "Add per-stage agent arrays in config.toml and automatic fallback when an agent quota is exhausted."
status: proposed
version: 0.1.0
---

# Overview

This change adds stage-level agent selection to `config.toml`, allowing each workflow stage (proposal, challenge, implementation, review) to define an ordered list of agents. When a stage runs, the system will try agents in order and automatically fall back to the next agent if the current one hits quota limits.

# Background

## Current State
- `config.toml` defines each agent (gemini, codex, claude) with model lists and defaults.
- The workflow has stages (proposal, challenge, implementation, review), but there is no configuration to map stages to a specific agent.

## Problem
- Users cannot configure which agent should run each stage.
- When a provider quota is exceeded, the workflow fails instead of switching to a backup agent.

# Proposed Solution

## New config.toml structure
Introduce phase-specific sections under `workflow.plan` and `workflow.implementation`, each with `executor` and `reviewer` arrays.

```toml
[workflow]
format_iterations = 2
planning_iterations = 3
implementation_iterations = 2
archive_iterations = 1

# Plan phase: proposal/spec generation and challenge review
[workflow.plan]
executor = ["gemini", "codex", "claude"]   # who generates proposal/specs
reviewer = ["codex", "claude"]             # who challenges/reviews

# Implementation phase: code implementation and code review
[workflow.implementation]
executor = ["codex", "gemini"]             # who implements code
reviewer = ["claude", "codex"]             # who reviews code
```

## Example full configuration
```toml
[workflow]
format_iterations = 2
planning_iterations = 3
implementation_iterations = 2
archive_iterations = 1

[workflow.plan]
executor = ["gemini", "codex"]
reviewer = ["codex", "claude"]

[workflow.implementation]
executor = ["codex", "gemini"]
reviewer = ["claude", "codex"]

[gemini]
command = "gemini"
default = "flash"
[[gemini.models]]
id = "flash"
model = "gemini-3-flash-preview"

[codex]
command = "codex"
default = "balanced"
[[codex.models]]
id = "fast"
model = "gpt-5.2-codex"

[claude]
command = "claude"
default = "balanced"
[[claude.models]]
id = "fast"
model = "haiku"
```

# Acceptance Criteria

- REQ-001: `config.toml` supports `workflow.{plan,implementation}.{executor,reviewer}` arrays with ordered agent names.
- REQ-002: If the first agent in a phase hits quota limits, the workflow retries using the next agent in the array.
- REQ-003: If all agents in a phase are exhausted, the workflow fails with a clear error that lists the attempted agents.
- REQ-004: `workflow.plan` and `workflow.implementation` are optional; defaults are applied when omitted.
- REQ-005: Existing agent model configuration remains backward compatible.
- REQ-006: Config validation rejects empty agent arrays, duplicate agent entries, and unknown agent names.
- REQ-007: Fallback retries pause for 5 seconds between agent switches to avoid rapid failure loops.
- REQ-008: Phase retries are idempotent: each run starts fresh and overwrites prior artifacts for that phase.

# Technical Design

## Config parsing changes
- Extend the config schema to include `workflow.plan` and `workflow.implementation` sections.
- Each phase contains `executor` and `reviewer` arrays, with defaults if omitted:
  - plan.executor: `["gemini"]` (proposal/spec generation)
  - plan.reviewer: `["codex"]` (challenge/review)
  - implementation.executor: `["codex"]` (code implementation)
  - implementation.reviewer: `["claude"]` (code review)
- Validate that each agent name exists as a top-level agent section (e.g., `[gemini]`, `[codex]`, `[claude]`).
- Validate that agent arrays are non-empty, contain no duplicates, and only include known agent names.
- Validation runs once at config load; errors stop startup before any workflow work begins.

## Quota error detection patterns
- Detect quota exhaustion primarily via non-zero exit codes from the agent command.
- When available, parse structured JSON error output for explicit quota/rate-limit codes as a secondary signal.
- Maintain a provider-agnostic list of regex patterns as a fallback confirmation, applied case-insensitively, e.g.:
  - `quota (exceeded|exhausted)`
  - `rate limit` / `rate-limit` / `too many requests`
  - `429` (HTTP status in error text)
  - `insufficient quota` / `usage limit`
- Allow per-agent overrides if needed later, but start with a shared pattern list.

## Fallback execution logic
- For each workflow stage, resolve the ordered `agents` list from config.
- Attempt execution with the first agent; if it succeeds, proceed.
- On failure, check if the error matches quota patterns.
  - If quota-related, log a warning and retry the stage with the next agent.
  - If not quota-related, propagate the error immediately.
- Track attempted agents and include them in any final failure message.
- Wait 5 seconds between agent switches to avoid rapid failure loops.
- Ensure retries are idempotent: each stage run starts fresh and overwrites prior artifacts for that stage, so no rollback is required.

# Out of Scope

- Automatic detection of non-quota transient errors (network errors, timeouts).
- Per-stage model overrides beyond the agent-level defaults.
- UI changes or new CLI flags beyond config usage.

# Testing

- Config validation: missing stage sections use defaults; empty arrays, duplicates, and unknown agents fail fast.
- Quota detection: non-zero exit code triggers fallback; JSON error output and regex patterns confirm quota classification.
- Retry behavior: stage switches agents in order with a 5-second backoff; all agents exhausted yields a clear error listing attempted agents.
- Idempotency: repeated stage execution overwrites prior artifacts without rollback or leftover partial outputs.

# Implementation Phases

## Phase 1 (This Change) - Infrastructure
- ✅ Config parsing with `workflow.stages.*` support (REQ-001, REQ-004)
- ✅ Config validation at all CLI entry points (REQ-006)
- ✅ Quota error detection module (supports REQ-002)
- ✅ AgentRunner API with fallback logic (REQ-002, REQ-003, REQ-007)
- ✅ AllAgentsExhaustedError with clear messaging (REQ-003)
- ✅ Backward compatibility maintained (REQ-005)

## Phase 2 (In Progress) - Integration
- ✅ Integrate AgentRunner into proposal_engine for proposal generation
- ✅ Add `run_llm_raw` method to ScriptRunner for quota detection
- ✅ Add raw MCP methods to Gemini/Codex/Claude orchestrators
- 🔲 Integrate AgentRunner into codex_orchestrator for challenge/review
- 🔲 Update implementation workflow to use AgentRunner
- 🔲 End-to-end testing with quota simulation
- 🔲 Integrate JSON quota detection into AgentRunner fallback path

## Known Limitations
- Only proposal generation phase currently uses AgentRunner fallback
- Challenge/review and implementation phases still use direct orchestrator calls
- Low-level CLI commands (proposal, tasks, spec, file) don't load config (they are service commands that don't use agent selection)
- Legacy `*_command` fields are parsed but not applied (backward compat for config loading only, not custom commands)
