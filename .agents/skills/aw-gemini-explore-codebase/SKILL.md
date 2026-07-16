---
name: aw:gemini:explore:codebase
description: Run Gemini headless to explore and analyze the codebase
user-invocable: true
---

# /aw:gemini:explore:codebase

Dispatches Gemini CLI headlessly to explore source code — architecture, dependencies, patterns, and implementations.

## Usage

```
/aw:gemini:explore:codebase "<prompt>"
```

## Instructions

1. Parse the user's prompt. If empty, ask the user what they want to explore.

2. Run Gemini CLI headlessly via Bash:

```bash
gemini -m gemini-3-flash-preview --output-format stream-json -p "Focus on source code under crates/ and src/. <prompt>"
```

3. Parse the streamed JSON output and present the findings to the user.

## Examples

```
# Map APIs
/aw:gemini:explore:codebase "Map all public APIs exposed by the agentic_workflow crate"

# Trace callers
/aw:gemini:explore:codebase "Find all callers of StateManager::load across the workspace"

# Architecture analysis
/aw:gemini:explore:codebase "Analyze the data flow from MCP tool call to state file update"

# Find patterns
/aw:gemini:explore:codebase "Find all files that use distributed_slice for CLI registration"
```

## AW CLI Drift & Defect Reporting

`aw` changes frequently. If this skill's documented invocation, result shape, or
semantics contradict the current `aw --help` output or CLI envelope, treat that
as a suspected AW defect; do not silently invent a compatibility command or
work around it.

Before reporting, reproduce the smallest failing command and capture the `aw`
version, exact command, expected result, actual stdout/stderr, and any relevant
envelope fields. Confirm the current surface with the relevant `aw <verb>
--help`; when working on AW itself, prefer a freshly built
`target/debug/aw` if the installed binary could be stale.

Once confirmed, report an AW-owned defect with `aw issue create --title "aw:
<short symptom>" "<reproduction and evidence>"`. Do not pass `--yes` unless
GitHub writes are already authorized. Expected validation failures or defects
owned by the target project belong in that project's tracker, not as AW bugs.
