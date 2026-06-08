---
id: nova-streaming
type: proposal
version: 1
created_at: 2026-01-31T02:49:58.095118+00:00
updated_at: 2026-01-31T02:49:58.095118+00:00
author: mcp
status: proposed
iteration: 1
summary: "Fix Claude provider and add full streaming support for cclab-nova-llm with Gemini support."
history:
  - timestamp: 2026-01-31T02:49:58.095118+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T02:51:51.302078+00:00
    agent: "unknown"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T02:52:07.468940+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
  - timestamp: 2026-01-31T02:55:28.866120+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T02:55:37.178690+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 9
  new_files: 1
affected_specs:
  - id: cclab-nova-llm-streaming
    path: specs/cclab-nova-llm-streaming.md
    depends: []---

<proposal>

# Change: nova-streaming

## Summary

Fix Claude provider and add full streaming support for cclab-nova-llm with Gemini support.

## Why

The current LLM provider implementation in cclab-nova-llm is incomplete, with a broken Claude provider and no support for Gemini. Streaming is also missing or inconsistent across providers and is not exposed to the Python interface, which is critical for real-time AI applications.

## What Changes

- Fix compilation errors and implement streaming in ClaudeProvider.
- Add GeminiProvider with full completion and streaming support.
- Add execute_stream to cclab-photon HttpClient to support streaming responses.
- Implement Python-compatible AsyncIterator for LLM streaming in cclab-nucleus.
- Expose Claude and Gemini providers to the Python API.

## Impact

- **Scope**: minor
- **Affected Files**: ~9
- **New Files**: ~1
- Affected specs:
  - `cclab-nova-llm-streaming` (no dependencies)
- Affected code: `crates/cclab-nova-llm/src/lib.rs`, `crates/cclab-nova-llm/src/provider.rs`, `crates/cclab-nova-llm/src/claude.rs`, `crates/cclab-nova-llm/src/openai.rs`, `crates/cclab-nova-llm/src/gemini.rs`, `crates/cclab-photon/src/client.rs`, `crates/cclab-photon/src/request.rs`, `crates/cclab-nucleus/src/agent/py_llm.rs`, `crates/cclab-nucleus/src/agent/mod.rs`
- **Breaking Changes**: The LLMProvider trait complete_stream method and the Python complete_stream interface will be updated to return a unified StreamResponse type instead of a raw Stream or raising NotImplementedError.

</proposal>
