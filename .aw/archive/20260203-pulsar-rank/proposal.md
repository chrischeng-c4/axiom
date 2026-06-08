---
id: pulsar-bm25
type: proposal
version: 1
created_at: 2026-01-30T05:09:54.154626+00:00
updated_at: 2026-01-30T05:09:54.154626+00:00
author: mcp
status: proposed
iteration: 1
summary: "Pure Rust implementation of BM25Okapi search ranking for the Pulsar ecosystem."
history:
  - timestamp: 2026-01-30T05:09:54.154626+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: major
  affected_files: 5
  new_files: 5
affected_specs:
  - id: pulsar-bm25-design
    path: specs/pulsar-bm25-design.md
    depends: []
---

<proposal>

# Change: pulsar-bm25

## Summary

Pure Rust implementation of BM25Okapi search ranking for the Pulsar ecosystem.

## Why

To provide high-quality text search and ranking capabilities within the Pulsar data science ecosystem without relying on heavy external dependencies. This enables building search engines, recommendation systems, and information retrieval tools directly in Rust with a familiar API.

## What Changes

- Create new crate crates/cclab-pulsar-bm25
- Implement Tokenizer trait for flexible text processing
- Implement BM25Okapi with batch indexing for TF-IDF and corpus statistics
- Support query scoring and top-K result retrieval
- Provide a Pythonic API compatible with rank-bm25

## Impact

- **Scope**: major
- **Affected Files**: ~5
- **New Files**: ~5
- Affected specs:
  - `pulsar-bm25-design` (no dependencies)
- Affected code: `crates/cclab-pulsar-bm25`

</proposal>
