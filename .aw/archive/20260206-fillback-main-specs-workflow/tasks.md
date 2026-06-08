---
id: fillback-main-specs-workflow
change_id: fillback-main-specs-workflow
type: tasks
version: 1
created_at: 2026-02-06T04:34:18.006089+00:00
updated_at: 2026-02-06T04:34:18.006089+00:00
proposal_ref: fillback-main-specs-workflow
summary:
  total: 4
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 4
layers:
  skill:
    task_count: 2
    estimated_files: 2
  registration:
    task_count: 2
    estimated_files: 2
history:
  - timestamp: 2026-02-06T04:34:18.006089+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 4 implementation tasks for change `fillback-main-specs-workflow`.

No Rust code changes needed — this is purely skill definition and template creation.

| Layer | Tasks | Status |
|-------|-------|--------|
| Skill Definition | 2 | 🔲 Pending |
| Registration | 2 | 🔲 Pending |

## 1. Skill Definition Layer

### Task 1.1: Create SKILL.md for fillback-main-specs

```yaml
id: 1.1
action: CREATE
status: pending
file: .claude/skills/cclab-genesis-fillback-main-specs/SKILL.md
spec_ref: fillback-skill-definition:R1,R2,R3,R4,R5,R6,R7
```

Create the main skill definition file with:
- Skill frontmatter (name, description, user-invocable: true)
- Full workflow orchestration instructions covering:
  - R1: Project structure detection (mono-repo vs single-project)
  - R2: Interactive component selection via AskUserQuestion
  - R3: Per-component code analysis pipeline (prism_symbols, analyze_code_for_spec)
  - R4: AI enrichment for rich spec generation (requirements, scenarios, Mermaid, API specs)
  - R5: Direct write to main specs via write_main_spec
  - R6: Existing spec detection and skip/overwrite logic
  - R7: Dynamic chunking for large codebases
- MCP tools reference section listing all tools used
- Usage examples for both mono-repo and non-mono-repo scenarios
- Step-by-step workflow instructions the LLM should follow

### Task 1.2: Create template copy for cclab init distribution

```yaml
id: 1.2
action: CREATE
status: pending
file: crates/cclab-genesis/templates/mainthread/skills/cclab-genesis-fillback-main-specs/SKILL.md
spec_ref: fillback-skill-definition:R8
depends_on: [1.1]
```

Copy the SKILL.md to the template directory so `cclab init` distributes it to new projects.
File should be identical to Task 1.1 output.

## 2. Registration Layer

### Task 2.1: Update project CLAUDE.md

```yaml
id: 2.1
action: MODIFY
status: pending
file: CLAUDE.md
spec_ref: fillback-skill-definition:R8
depends_on: [1.1]
```

Add the new skill to the workflow table in CLAUDE.md between the `<!-- cclab:gen:start -->` and `<!-- cclab:gen:end -->` markers. Add under "Utility Skills" section:
- `/cclab:genesis:fillback-main-specs` | Fillback: generate rich specs from existing code

### Task 2.2: Update template CLAUDE.md

```yaml
id: 2.2
action: MODIFY
status: pending
file: crates/cclab-genesis/templates/mainthread/CLAUDE.md
spec_ref: fillback-skill-definition:R8
depends_on: [2.1]
```

Mirror the same skill registration in the template CLAUDE.md for cclab init distribution.

</tasks>
