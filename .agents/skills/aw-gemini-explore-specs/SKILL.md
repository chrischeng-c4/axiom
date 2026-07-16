---
name: aw:gemini:explore:specs
description: Run Gemini headless to explore project specs and knowledge base
user-invocable: true
---

# /aw:gemini:explore:specs

Dispatches Gemini CLI headlessly to explore `apps/agentic-workflow/tech-design/`, `cclab/knowledge/`, and `/tmp/aw/workspaces/<workspace>/changes/` — AW specs, knowledge base, and change artifacts.

## Usage

```
/aw:gemini:explore:specs "<prompt>"
```

## Instructions

1. Parse the user's prompt. If empty, ask the user what they want to explore.

2. Run Gemini CLI headlessly via Bash:

```bash
gemini -m gemini-3-flash-preview --output-format stream-json -p "Focus on files under apps/agentic-workflow/tech-design/, cclab/knowledge/, and /tmp/aw/workspaces/<workspace>/changes/. <prompt>"
```

3. Parse the streamed JSON output and present the findings to the user.

## Examples

```
# Find specs related to a topic
/aw:gemini:explore:specs "Find all specs related to the Agentic Workflow state machine"

# Understand a change
/aw:gemini:explore:specs "Summarize the agentic-workflow merge change — what was decided and why"

# Cross-reference specs
/aw:gemini:explore:specs "Which specs reference StatePhase and what do they say about valid transitions?"

# Knowledge base search
/aw:gemini:explore:specs "What conventions does the knowledge base define for crate splitting?"
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
