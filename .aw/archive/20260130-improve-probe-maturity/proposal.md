---
id: improve-probe-maturity
type: proposal
version: 1
created_at: 2026-01-28T16:58:54.781523+00:00
updated_at: 2026-01-28T16:58:54.781523+00:00
author: mcp
status: proposed
iteration: 1
summary: "Upgrade cclab-probe maturity to 95% with Plugin System, Fixture DI, and Agent Eval framework."
history:
  - timestamp: 2026-01-28T16:58:54.781523+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 15
  new_files: 0
affected_specs:
  - id: plugin-system
    path: specs/plugin-system.md
    depends: []
  - id: fixture-di-integration
    path: specs/fixture-di-integration.md
    depends: []
  - id: fixture-di-integration-alt
    path: specs/fixture-di-integration-alt.md
    depends: []
  - id: expect-api-reference
    path: specs/expect-api-reference.md
    depends: []
  - id: agent-eval
    path: specs/agent-eval.md
    depends: []
---

<proposal>

# Change: improve-probe-maturity

## Summary

Upgrade cclab-probe maturity to 95% with Plugin System, Fixture DI, and Agent Eval framework.

## Why

cclab-probe currently lacks the extensibility, deep fixture integration, and AI-specific evaluation tools required for a full pytest alternative in the cclab ecosystem. This upgrade is necessary to support complex testing workflows and systematic evaluation of AI agents.

## What Changes

- Implement a hook-based plugin system (inspired by pluggy/pytest) for test lifecycle management.
- Wire the Fixture DI system into the TestRunner to support automatic dependency injection.
- Implement a comprehensive Agent Evaluation framework with LLM-as-a-Judge and cost tracking.
- Expand the fluent Expect assertion API with JSON Path and Option matchers.
- Add comprehensive test coverage for plugin priorities and async fixtures.

## Impact

- **Scope**: minor
- **Affected Files**: ~15
- **New Files**: ~0
- Affected specs:
  - `plugin-system` (no dependencies)
  - `fixture-di-integration` (no dependencies)
  - `fixture-di-integration-alt` (no dependencies)
  - `expect-api-reference` (no dependencies)
  - `agent-eval` (no dependencies)

</proposal>
