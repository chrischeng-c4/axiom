---
id: pulsar-jieba
type: proposal
version: 1
created_at: 2026-01-30T04:35:52.243666+00:00
updated_at: 2026-01-30T04:35:52.243666+00:00
author: mcp
status: proposed
iteration: 1
summary: "Pure Rust Chinese tokenization library (jieba alternative) with Pythonic API and proper crate structure"
history:
  - timestamp: 2026-01-30T04:35:52.243666+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-30T04:36:05.426984+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-30T04:36:17.293855+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 8
  new_files: 6
affected_specs:
  - id: pulsar-jieba-design
    path: specs/pulsar-jieba-design.md
    depends: []
  - id: pulsar-jieba-interfaces
    path: specs/pulsar-jieba-interfaces.md
    depends: []
  - id: pulsar-jieba-integration
    path: specs/pulsar-jieba-integration.md
    depends: []---

<proposal>

# Change: pulsar-jieba

## Summary

Pure Rust Chinese tokenization library (jieba alternative) with Pythonic API and proper crate structure

## Why

To provide the Pulsar data science ecosystem with a high-performance, pure-Rust Chinese NLP capability. By avoiding external Rust dependencies (beyond workspace defaults), we ensure a lightweight and stable foundation for text analysis that can be seamlessly used from both Rust and Python. The revision ensures proper crate structure and spec organization.

## What Changes

- Create new crate `cclab-pulsar-jieba` for core Chinese NLP logic
- Implement Precise, Full, and Search segmentation modes using a Trie-based DAG
- Embed default Chinese dictionary for zero-configuration startup
- Implement Viterbi algorithm with HMM for unknown word recognition
- Implement TF-IDF for keyword extraction and HMM-based POS tagging
- Integrate into `cclab-nucleus` under a new `pulsar` feature to provide a Pythonic API
- Update `cclab/specs/crate-map.md` to reflect the new Pulsar crate

## Impact

- **Scope**: minor
- **Affected Files**: ~8
- **New Files**: ~6
- Affected specs:
  - `pulsar-jieba-design` (no dependencies)
  - `pulsar-jieba-interfaces` (no dependencies)
  - `pulsar-jieba-integration` (no dependencies)
- Affected code: `crates/cclab-pulsar-jieba`, `cclab/specs/crate-map.md`, `crates/cclab-nucleus/Cargo.toml`, `crates/cclab-nucleus/src/lib.rs`

</proposal>
