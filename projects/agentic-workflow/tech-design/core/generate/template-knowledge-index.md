---
id: knowledge-index-template
type: spec
title: "Knowledge Index Template"
version: 1
spec_type: utility
spec_group: cclab-sdd
created_at: 2026-02-23T00:00:00+00:00
updated_at: 2026-02-23T00:00:00+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Codegen TDs support CB lifecycle generation and regenerable artifact production."
---

# Knowledge Index Template

## Overview
<!-- type: overview lang: markdown -->

The knowledge index template bootstraps the `cclab/knowledge/` directory with a starter `index.md` file. It explains the knowledge base structure, numbering conventions, and CLI tool integration.

**Knowledge vs Specs**:
- **Specs** (`.aw/tech-design/`) are prescriptive — they define what should be built
- **Knowledge** (`cclab/knowledge/`) is descriptive — it documents how the system currently works

The index file serves as a README for the knowledge directory and is the first file users see when exploring the knowledge base.

**Template source**: `crates/cclab-sdd/templates/knowledge/index.md`

## Template
<!-- type: doc lang: markdown -->

```markdown
# Knowledge Base

This directory contains descriptive documentation about how this system works.

- **Specs** define what should be built (prescriptive)
- **Knowledge** documents how it currently works (descriptive)

Structure

Use numbered prefixes for ordering:
- `00-09`: System-level (architecture, principles)
- `10+`: Domain modules

Example:
` ` `
knowledge/
  00-architecture/
    index.md
    01-overview.md
    02-design-principles.md
  10-auth/
    index.md
    01-oauth-flow.md
` ` `

Contents

<!-- Add entries as you document the system -->

Usage

LLM tools can read knowledge via CLI:
- `list_knowledge` - List all knowledge files
- `read_knowledge` - Read specific file
```

## Installation
<!-- type: doc lang: markdown -->

### R1 - Compile-Time Embedding

```yaml
id: R1
priority: high
status: draft
```

The template is embedded into the binary via `include_str!()` as `KNOWLEDGE_INDEX_TEMPLATE` in `init.rs:20`. The installer does NOT read from the filesystem at runtime.

### R2 - Create-If-Missing Semantics

```yaml
id: R2
priority: high
status: draft
```

Installation behavior depends on context:

- **Fresh install** (`cclab init` on uninitialized project): always creates `cclab/knowledge/index.md` with the template content.
- **Update** (`cclab init` on already-initialized project): only creates `cclab/knowledge/index.md` if the file does not already exist. Never overwrites user content.

This protects user-curated knowledge from being reset on upgrade.

### R3 - Parent Directory Creation

```yaml
id: R3
priority: medium
status: draft
```

The `cclab/knowledge/` directory is created via `std::fs::create_dir_all()` before writing the index file. This ensures the directory exists regardless of whether it was manually deleted or never created.
