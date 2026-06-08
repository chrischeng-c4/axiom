---
id: link-fetching
type: spec
title: "Nebula Link Fetching Migration"
version: 1
spec_type: integration
spec_group: cclab-nebula
merge_strategy: new
created_at: 2026-02-03T09:21:46.829965+00:00
updated_at: 2026-02-03T09:21:46.829965+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: sequence
      title: "Batch Link Fetching Flow"
history:
  - timestamp: 2026-02-03T09:21:46.829965+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Nebula Link Fetching Migration

## Overview

Offload the complex batch link fetching logic to Rust. Python will provide the data and schema (LinkField definitions), and Rust will efficiently query and populate the links.

## Requirements

### R1 - Schema Extraction

```yaml
id: R1
priority: medium
status: draft
```

Python must extract `Link` and `BackLink` definitions from the model and convert them to `PyLinkField` objects.

### R2 - Replace Implementation

```yaml
id: R2
priority: medium
status: draft
```

Python `QueryBuilder` must replace `_batch_fetch_links_for_list` with a call to `cclab._nebula.fetch_links_batched`.

### R3 - Recursive Fetching

```yaml
id: R3
priority: medium
status: draft
```

The Rust implementation must recursively fetch links up to the specified depth.

## Acceptance Criteria

### Scenario: Fetch Links

- **GIVEN** A list of documents with unresolved `Link` fields
- **WHEN** User requests `fetch_links=True`
- **THEN** The Rust function populates the fields with the fetched document data.

### Scenario: Deep Fetch

- **GIVEN** A `fetch_links_depth` > 1
- **WHEN** User queries with depth
- **THEN** Rust recursively fetches nested links.

## Diagrams

### Batch Link Fetching Flow

```mermaid
sequenceDiagram
    participant Python as Python Layer
    participant Rust as Rust Link Logic
    participant MongoDB as MongoDB
    Python->>Python: Inspect Model for Link Fields
    Python->>Python: Construct List[PyLinkField]
    Python->>Rust: Call fetch_links_batched(docs, fields)
    Rust->>Rust: Analyze docs for refs
    Rust->>MongoDB: Batch fetch referenced docs (per collection)
    MongoDB->>Rust: Return referenced docs
    Rust->>Rust: Distribute docs into source docs
    Rust->>Python: Return updated docs
```

</spec>
