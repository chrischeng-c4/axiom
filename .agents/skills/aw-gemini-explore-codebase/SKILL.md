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
