---
change_id: genesis-186-28
type: knowledge_context
created_at: 2026-02-14T03:21:03.367851+00:00
updated_at: 2026-02-14T03:21:03.367851+00:00
iteration: 2
complexity: high
stage: knowledge
scanned_categories:
  - spec-to-code
  - mcp-configuration
  - agent-skills
---

# Knowledge Context

## Relevant Documents

- **knowledge:spec-to-code/index.md**
  - summary: Overview of the spec-to-code pipeline and the core spec models.
  - relevant sections: Spec Model, Code Generator Contract
- **knowledge:spec-to-code/spec-model.md**
  - summary: Detailed mapping of 6 core spec types to code structures. Emphasizes agnostic descriptions.
  - relevant sections: Spec Catalog, Sequence Plus, Flowchart Plus, Requirement Plus
- **knowledge:spec-to-code/code-generator-contract.md**
  - summary: Defines how generators should map specs to framework-specific code and identifies the current gap in multi-spec consumption.
  - relevant sections: Generator Responsibilities, Inference Rules, Current Gap
- **knowledge:40-mcp/index.md**
  - summary: Introduction to dynamic MCP configuration for Genesis workflow stages.
  - relevant sections: Dynamic Configuration
- **knowledge:40-mcp/dynamic-config.md**
  - summary: Implementation details for stage-specific MCP tool filtering to reduce LLM cognitive load.
  - relevant sections: Solution Architecture, Tool Sets by Stage
- **knowledge:30-claude/skills.md**
  - summary: Guide on creating and using Claude Code Skills for capability extension.
  - relevant sections: How Skills Work, Creating Your First Skill

## Patterns

- **Spec-Driven Development (SDD)** (source: knowledge:spec-to-code/spec-model.md)
  - Agnostic Technical Design using 6 core spec types. Example: A Sequence Plus message `AuthHandler->>TokenService: validate(token)` maps to a function signature in the TokenService module.
- **Dynamic MCP Configuration** (source: knowledge:40-mcp/dynamic-config.md)
  - Filtering available MCP tools based on workflow stage. Example: Running `cclab server --tools implement` reduces the 22 core tools down to 4 (read_all_requirements, read_implementation_summary, list_changed_files, read_file).
- **Agent Skills Extension** (source: knowledge:30-claude/skills.md)
  - Specialized instructions via markdown files with YAML metadata. Example: A `SKILL.md` file with `name: explaining-code` and `description: Explains code with visual diagrams.` automatically triggers when a user asks how code works.

## Pitfalls

- Exposing too many tools to the LLM increases cognitive load and token usage.
- Including implementation-specific code in specs instead of high-level abstractions.
- Existing generators only consume API Specs, neglecting the richer context from Sequence+, Flowchart+, etc.
