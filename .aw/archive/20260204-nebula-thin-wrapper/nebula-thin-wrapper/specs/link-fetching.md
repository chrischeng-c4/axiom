---
id: link-fetching
type: spec
title: "Link Fetching (Batched)"
version: 1
spec_type: algorithm
created_at: 2026-02-03T10:02:41.297827+00:00
updated_at: 2026-02-03T10:02:41.297827+00:00
requirements:
  total: 7
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "Batched Link Fetch Flow (Semantic)"
history:
  - timestamp: 2026-02-03T10:02:41.297827+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-03T10:02:56.092974+00:00
    agent: "codex:deep"
    tool: "revise_spec"
    action: "revised"
  - timestamp: 2026-02-03T10:03:16.284096+00:00
    agent: "codex:max"
    tool: "review_spec"
    action: "reviewed"---

<spec>

# Link Fetching (Batched)

## Overview

Define the Rust-backed batched link fetching flow for Nebula documents, preserving the Python API while delegating forward-link resolution to `fetch_links_batched` and keeping BackLink handling in Python. Depth validation, schema extraction, and error handling are enforced in Python before and after the Rust call.

## Requirements

### R1 - Depth Validation

```yaml
id: R1
priority: high
status: draft
```

Link fetching must treat `depth <= 0` as a no-op and validate that requested depth is within the Rust batcher limits (1..=5). Depth outside this range must raise `ValueError` with a clear message before calling the Rust binding.

### R2 - Link Metadata Extraction

```yaml
id: R2
priority: high
status: draft
```

The Python layer must derive a list of link descriptors for each model using type annotations: forward `Link[T]` (single) and list variants (e.g., `list[Link[T]]`) should map to `LinkField.link` or `LinkField.link_list`, while `BackLink[T]` maps to `LinkField.back_link`. Each descriptor must include the target collection name resolved from the target document class.

### R3 - Batched Forward-Link Fetching

```yaml
id: R3
priority: high
status: draft
```

For forward links, the Python layer must call `fetch_links_batched(docs, link_fields, max_depth)` with docs serialized to dicts containing `_id` plus link fields. Link references must be passed in supported formats (ObjectId, DBRef-style dict, or string ObjectId) so the Rust extractor can resolve them.

### R4 - Result Mapping To Python Documents

```yaml
id: R4
priority: high
status: draft
```

After `fetch_links_batched` returns, the Python layer must map fetched subdocuments back onto the original `Document` instances, replacing forward link fields with `Link` objects that hold the fetched document and `document_class` set to the target class. Pre-fetched links must remain unchanged; missing targets must leave the existing ref intact.

### R5 - BackLink Handling

```yaml
id: R5
priority: medium
status: draft
```

BackLink fields must be populated via the existing Python `_fetch_backlink_field` logic after forward links are resolved, respecting the requested depth and preserving BackLink semantics.

### R6 - Compatibility With Query Results

```yaml
id: R6
priority: medium
status: draft
```

QueryBuilder list results and `Document.fetch_all_links` must both route through the Rust batcher for forward links, ensuring the same behavior and ordering as the prior Python implementation.

### R7 - Error Surface And Stability

```yaml
id: R7
priority: medium
status: draft
```

Batch link fetching failures (invalid inputs or Mongo errors) must surface as Python exceptions without leaving partially mutated documents; on error, documents should either remain unchanged or be updated only with successfully fetched links.

## Acceptance Criteria

### Scenario: No-Op Depth

- **GIVEN** a document with forward links and depth=0
- **WHEN** link fetching is invoked
- **THEN** no Rust call is made and the document remains unchanged

### Scenario: Depth Upper Bound Validation

- **GIVEN** a request with depth=6
- **WHEN** link fetching is invoked
- **THEN** a ValueError is raised before calling Rust, explaining the supported depth range

### Scenario: Link Field Descriptor Extraction

- **GIVEN** a model with `Link[User]`, `list[Link[Tag]]`, and `BackLink[Post]` annotations
- **WHEN** link metadata extraction runs
- **THEN** the descriptors include `link`, `link_list`, and `back_link` entries with target collection names derived from the referenced document classes

### Scenario: Single Forward Link Batch

- **GIVEN** two Post documents each with `author` stored as an ObjectId reference
- **WHEN** batch link fetching runs with depth=1
- **THEN** one batched query per target collection occurs and each Post.author becomes a Link holding the fetched User

### Scenario: List Forward Links

- **GIVEN** a Post with `tags: list[Link[Tag]]` stored as object id strings
- **WHEN** batch link fetching runs
- **THEN** each list entry is replaced by the fetched Tag document at the same index

### Scenario: BackLink Preservation

- **GIVEN** a User document with BackLink posts and depth=1
- **WHEN** batch link fetching runs
- **THEN** forward links are populated via Rust and BackLink posts are populated via `_fetch_backlink_field`

### Scenario: Missing Target Document

- **GIVEN** a document referencing a non-existent ObjectId
- **WHEN** batch link fetching runs
- **THEN** the link field remains as an unresolved reference and no exception is raised

### Scenario: Rust Failure Does Not Partially Mutate

- **GIVEN** the Rust batcher returns an error after inspecting input documents
- **WHEN** batch link fetching runs
- **THEN** the error is raised as a Python exception and the original documents remain unchanged

## Diagrams

### Batched Link Fetch Flow (Semantic)

```mermaid
flowchart LR
    start(Start)
    check_noop{Depth <= 0?} 
    noop_end(No-op)
    check_max{Depth > 5?} 
    error[Raise ValueError]
    collect[Serialize docs + link fields]
    call_rust[fetch_links_batched]
    apply[Map fetched docs to Links]
    backlinks[Fetch BackLinks (Python)]
    done(Done)
    start --> check_noop
    check_noop -->|yes| noop_end
    check_noop -->|no| check_max
    check_max -->|yes| error
    check_max -->|no| collect
    collect --> call_rust
    call_rust --> apply
    apply --> backlinks
    backlinks --> done
```

<semantic-data>

```json
{
  "edges": [
    {
      "from": "check_noop",
      "semantic": {
        "condition": "depth <= 0"
      },
      "to": "noop_end"
    },
    {
      "from": "check_max",
      "semantic": {
        "condition": "depth > 5",
        "is_error_path": true
      },
      "to": "error"
    },
    {
      "from": "check_max",
      "semantic": {
        "condition": "1..=5"
      },
      "to": "collect"
    }
  ],
  "nodes": [
    {
      "id": "start",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "check_noop",
      "semantic": {
        "code_pattern": "depth <= 0",
        "type": "condition"
      }
    },
    {
      "id": "noop_end",
      "semantic": {
        "output": {
          "name": "documents",
          "type": "List[Document]"
        },
        "type": "end"
      }
    },
    {
      "id": "check_max",
      "semantic": {
        "code_pattern": "depth > 5",
        "type": "condition"
      }
    },
    {
      "id": "error",
      "semantic": {
        "error": {
          "code": 400,
          "message": "depth must be 1..=5"
        },
        "type": "raise_error"
      }
    },
    {
      "id": "collect",
      "semantic": {
        "code_pattern": "serialize _id + link fields; build LinkField list",
        "input": {
          "name": "documents",
          "source": "python",
          "type": "List[Document]"
        },
        "output": {
          "name": "payload",
          "type": "BatchedLinkPayload"
        },
        "type": "transform"
      }
    },
    {
      "id": "call_rust",
      "semantic": {
        "input": {
          "name": "payload",
          "type": "BatchedLinkPayload"
        },
        "method": "POST",
        "output": {
          "name": "fetched_docs",
          "type": "List[Doc]"
        },
        "type": "api_call",
        "url": "pyo3.fetch_links_batched"
      }
    },
    {
      "id": "apply",
      "semantic": {
        "code_pattern": "replace forward link refs with Link(document, document_class)",
        "input": {
          "name": "fetched_docs",
          "type": "List[Doc]"
        },
        "output": {
          "name": "documents",
          "type": "List[Document]"
        },
        "type": "transform"
      }
    },
    {
      "id": "backlinks",
      "semantic": {
        "code_pattern": "_fetch_backlink_field per BackLink",
        "input": {
          "name": "documents",
          "type": "List[Document]"
        },
        "output": {
          "name": "documents",
          "type": "List[Document]"
        },
        "type": "transform"
      }
    },
    {
      "id": "done",
      "semantic": {
        "output": {
          "name": "documents",
          "type": "List[Document]"
        },
        "type": "end"
      }
    }
  ]
}
```

</semantic-data>

</spec>
