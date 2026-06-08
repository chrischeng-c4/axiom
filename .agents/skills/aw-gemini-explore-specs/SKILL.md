---
name: aw:gemini:explore:specs
description: Run Gemini headless to explore project specs and knowledge base
user-invocable: true
---

# /aw:gemini:explore:specs

Dispatches Gemini CLI headlessly to explore `projects/agentic-workflow/tech-design/`, `cclab/knowledge/`, and `.aw/changes/` — AW specs, knowledge base, and change artifacts.

## Usage

```
/aw:gemini:explore:specs "<prompt>"
```

## Instructions

1. Parse the user's prompt. If empty, ask the user what they want to explore.

2. Run Gemini CLI headlessly via Bash:

```bash
gemini -m gemini-3-flash-preview --output-format stream-json -p "Focus on files under projects/agentic-workflow/tech-design/, cclab/knowledge/, and .aw/changes/. <prompt>"
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
