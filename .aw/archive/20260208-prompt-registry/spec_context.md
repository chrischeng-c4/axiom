---
change_id: prompt-registry
type: spec_context
created_at: 2026-02-08T17:42:38.215928+00:00
updated_at: 2026-02-08T17:42:38.215928+00:00
iteration: 1
complexity: medium
stage: spec
scanned_groups:
  - genesis
---

# Spec Context

## Relevant Specs

- **prompt-registry** (group: genesis)
  - relevance: high
  - reason: Previous iteration spec — inlined all prompts into flow files. Current change extends this by DRY-ing the prompt text via shared functions.
  - key sections: R1 - folder structure (done), R2 - agent_prompt population (done), R3 - delete src/prompts/ (done)
- **agent-tool** (group: genesis)
  - relevance: medium
  - reason: genesis_agent is referenced in executor chain and prompt constants. Prompt registry constants (GENESIS_AGENT_TOOL) already reference it.
  - key sections: R1 - Tool Parameters, R3 - Action Templates
- **impl-change-tool** (group: genesis)
  - relevance: low
  - reason: Legacy spec for deprecated impl_change tool. Superseded by run_change/implement.rs flow.
  - key sections: R5 - Response Format

## Dependencies

- prompt-registry depends on run_change module structure
- prompt constants reference genesis_agent tool name

## Gaps

- No spec for the shared prompt functions/constants pattern
- No spec for the unified prompt + instructions response format (implemented but not spec'd)
