---
number: 1126
title: "feat(sdd): agent execution timeout — prevent infinite agent runs"
state: open
labels: [type:enhancement, priority:p2, crate:sdd]
group: "phase-advance-and-timeout"
---

# #1126 — feat(sdd): agent execution timeout — prevent infinite agent runs

## Problem

Delegated agents (`claude-agent:*`) have no execution timeout. When an agent gets stuck (e.g., in a retry loop, waiting for a locked resource, or exploring irrelevant code), it runs indefinitely until manually killed.

## Observed

- Implementation agents ran for 30+ minutes without producing useful output
- Reference-context review agent completed with exit_code 0 but didn't write the required artifact
- Only remedy was `pkill -f sdd-change-implementation`

## Proposed

Add configurable timeout to agent dispatch:
1. Default timeout per action type (e.g., reference_context: 5m, change_spec: 10m, implementation: 15m)
2. Configurable in `cclab/config.toml` under `[workflow.timeouts]`
3. On timeout: kill agent process, log timeout event, advance to retry or terminal failure
4. Consider adding a `--timeout` flag to `sdd_delegate_agent` MCP tool
