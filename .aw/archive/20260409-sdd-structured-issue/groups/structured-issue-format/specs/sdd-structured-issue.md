---
id: sdd-structured-issue
main_spec_ref: crates/sdd/logic/structured-issue.md
merge_strategy: new
fill_sections: [overview, schema, cli, changes]
filled_sections: [overview, schema, cli, changes]
create_complete: true
---

# Sdd Structured Issue

## Overview

<!-- type: overview lang: markdown -->

Structured issue format for SDD that absorbs early phases (restructure_input, pre/post_clarifications, reference_context) into issue authoring.

### Problem

SDD phases 2-3 and 7 re-derive information that a well-structured issue should already contain. This adds ~6 mainthread round-trips per change.

### Solution

| Issue Section | Replaces SDD Phase |
|---------------|--------------------|
| `## Problem` + `## Requirements` | restructure_input (phase 2) |
| `## Key Decisions` | pre_clarifications (phase 3) |
| `## Scope` + `## Acceptance Criteria` | post_clarifications (phase 7) |
| `## Reference Context` | reference_context (phase 4-6) |

### Components

| Component | Location | Responsibility |
|-----------|----------|----------------|
| Issue section parser | `crates/sdd/src/services/issue_parser.rs` | Extract structured sections from issue markdown |
| init_change update | `crates/sdd/src/tools/init_change.rs` | Detect structured issue, auto-generate artifacts, skip to post_clarifications_created |
| `score issues enrich` | `projects/score/cli/src/issues.rs` | Run reference_context agent to fill Reference Context section |
| SKILL.md update | `projects/score/cli/templates/mainthread/skills/` | Mainthread detects structured issues, skips early phases |

### Constraints

- Backward compat: unstructured issues fall back to current flow
- No new state machine states — skip by advancing phase directly
- Reference Context is agent-filled via `score issues enrich`, not manual
## Requirements
<!-- type: requirements lang: markdown -->

<!-- TODO -->

## Scenarios
<!-- type: scenarios lang: markdown -->

<!-- TODO -->

## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

<!-- type: changes lang: markdown -->

### 1. `crates/sdd/src/services/issue_parser.rs` — New: issue section parser

```rust
pub struct StructuredIssue {
    pub problem: String,
    pub requirements: Vec<Requirement>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub scope: IssueScope,
    pub key_decisions: Vec<Decision>,
    pub reference_context: Option<IssueReferenceContext>,
}

pub fn parse_structured_issue(body: &str) -> Option<StructuredIssue>
pub fn is_structured_issue(body: &str) -> bool
```

Parse markdown body by splitting on `## ` headers. Extract R1/R2/AC1/D1 patterns from list items.

### 2. `crates/sdd/src/tools/init_change.rs` — Update: structured issue detection + phase skip

In the init_change handler, after creating STATE.yaml:

```rust
// After writing user_input.md and STATE.yaml
if let Some(issue_slug) = extract_issue_slug(&description) {
    if let Some(issue) = load_issue(issue_slug) {
        if let Some(structured) = parse_structured_issue(&issue.body) {
            // Auto-generate artifacts from structured issue
            write_requirements_md(change_dir, &structured);
            write_pre_clarifications_md(change_dir, &structured.key_decisions);
            write_post_clarifications_md(change_dir, &structured.scope, &structured.acceptance_criteria);
            if let Some(ref_ctx) = &structured.reference_context {
                write_reference_context_md(change_dir, ref_ctx);
            }
            // Skip phases: advance directly to post_clarifications_created
            state.phase = StatePhase::PostClarificationsCreated;
            state.save()?;
        }
    }
}
```

### 3. `projects/score/cli/src/issues.rs` — New: `enrich` subcommand

Add `Enrich` variant to the issues CLI:

```rust
#[derive(Subcommand)]
enum IssuesCommands {
    // ... existing
    Enrich {
        slug: String,
        #[arg(long)]
        dry_run: bool,
    },
}
```

Implementation: read issue → validate required sections → run reference_context exploration → write `## Reference Context` back.

### 4. `projects/score/cli/templates/mainthread/skills/score-run-change/SKILL.md` — Update: skip detection

Add to the Entry Point section:

```markdown
### 4. Structured Issue Detection

After init_change, if the response shows `current_phase: post_clarifications_created` immediately
(skipped phases 2-3, 7), continue the loop from that phase. The early phases were auto-filled
from the structured issue.
```

No mainthread logic change needed — the loop already handles any phase. The skip is transparent.
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->


## Schema

<!-- type: schema lang: json -->

### Structured Issue Sections

Required sections detected by `## Header` matching:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "StructuredIssue",
  "description": "Parsed sections from a structured issue markdown file",
  "type": "object",
  "properties": {
    "problem": {
      "type": "string",
      "description": "## Problem section body — what's wrong or needed"
    },
    "requirements": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": { "type": "string", "pattern": "^R\\d+$" },
          "text": { "type": "string" },
          "priority": { "type": "string", "enum": ["high", "medium", "low"] }
        },
        "required": ["id", "text"]
      }
    },
    "acceptance_criteria": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": { "type": "string", "pattern": "^AC\\d+$" },
          "text": { "type": "string" }
        },
        "required": ["id", "text"]
      }
    },
    "scope": {
      "type": "object",
      "properties": {
        "in_scope": { "type": "string" },
        "out_of_scope": { "type": "string" }
      }
    },
    "key_decisions": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": { "type": "string", "pattern": "^D\\d+$" },
          "text": { "type": "string" }
        },
        "required": ["id", "text"]
      }
    },
    "reference_context": {
      "type": "object",
      "properties": {
        "specs": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "path": { "type": "string" },
              "relevance": { "type": "string", "enum": ["high", "medium", "low"] },
              "key_requirements": { "type": "string" }
            }
          }
        },
        "spec_plan": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "spec_id": { "type": "string" },
              "action": { "type": "string", "enum": ["create", "modify"] },
              "main_spec_ref": { "type": "string" },
              "sections": { "type": "array", "items": { "type": "string" } }
            }
          }
        }
      }
    }
  },
  "required": ["problem", "requirements", "scope"]
}
```

### Detection Logic

```rust
// Structured issue = has all required section headers
fn is_structured_issue(body: &str) -> bool {
    let required = ["## Problem", "## Requirements", "## Scope"];
    required.iter().all(|h| body.contains(h))
}
```

Optional sections (`## Acceptance Criteria`, `## Key Decisions`, `## Reference Context`) enhance skip quality but are not required.


## CLI

<!-- type: cli lang: yaml -->

```yaml
commands:
  score:
    issues:
      enrich:
        description: "Fill Reference Context section of a structured issue by exploring specs and codebase"
        args:
          slug:
            type: string
            required: true
            description: "Issue slug (filename stem from .score/issues/)"
        flags:
          --dry-run:
            type: bool
            default: false
            description: "Print Reference Context without writing to issue file"
        behavior:
          - Read issue from .score/issues/open/{slug}.md
          - Validate it has ## Problem and ## Requirements sections
          - Run reference_context exploration (reuse sdd-reference-context agent logic)
          - Parse requirements to identify affected crates and spec areas
          - Build spec table (path, relevance, key_requirements) and spec_plan
          - Write ## Reference Context section back to the issue file
        output:
          success: "Enriched {slug} with {n} specs and {m} spec_plan entries"
          error: "Issue missing required sections: ## Problem, ## Requirements"
```

# Reviews
