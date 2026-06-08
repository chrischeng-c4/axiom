# Specs Changelog

All notable changes to specifications will be documented here.

## [Unreleased]

## 2026-04-24: jet-wasm Debugging Experience (jet-wasm-debug-fillback)
Fillback spec pass documenting the debug-surface + list-render
milestone shipped in commits `ae2be7d7` → `7a372378`. Previous
commits bypassed SDD; this entry retroactively captures the
architecture decisions they made.
- **New specs**:
  - `crates/jet/wasm-renderer/debug-bridge.md` — `JetDebug` surface
    (elementTree / layoutTree / fiberTree / paintOps / hookValues /
    pickAt / highlight / forceRerender), `window.__jet_debug`
    registration, serialization mirror types (`DebugElement`,
    `DebugLayoutTree`, `DebugPaintOp`, …), `debug` feature gate.
  - `crates/jet/wasm-renderer/browser-cli.md` — `jet browser`
    subcommand tree (launch / debug / tree / pick / hooks /
    highlight / frame / screenshot / eval / tsx), session file
    format at `.jet/browser-session.json`, `CHROME_PATH` env var
    plumbing, pick-listener capture flow.
  - `crates/jet/wasm-renderer/wasm-dev-server.md` — `jet dev --wasm`
    axum static server + notify watcher + 150ms-debounced
    auto-rebuild + `--debug` profile plumbing.
- **Updated specs**:
  - `crates/jet/wasm-renderer/architecture.md` — added
    `Element::Fragment` enum variant, debug-feature + event-loop
    class diagram, reshuffled phased-delivery status table
    (phases 4/5 now shipped, 5a/5b/5c new).
  - `crates/jet/wasm-renderer/transpiler.md` — added
    `transpile_with_source` + `PositionMap` side-car,
    non-Copy `.clone()` rules (prop-field init + setter bare-ident
    arg), always-destructure prop prelude, `[...Array(n)].map()`
    lowering.
  - `crates/jet/wasm-renderer/paint-runtime.md` — added Fragment
    transparent walk, click event loop with Rc-shared LayoutTree,
    last-frame op cache, debug highlight overlay pass.
  - `crates/jet/wasm-renderer/subset.md` — added "verified
    features" table keyed to `examples/<name>-demo/` +
    `crates/jet/tests/*_debug.rs` pairs.
- Related commits: ae2be7d7, b02f5bee, d022e4ad, d8ec7d41, 7a372378

## 2026-02-16: Spec Directory Consolidation (sdd-spec-consolidation)
Consolidated all specs from the former `cclab-aurora` and `cclab-genesis` directories into `cclab-sdd/`, completing the crate unification at the spec level.
- **Aurora → generate/**: Moved 15 diagram/code generation specs into `cclab-sdd/generate/`, renamed all `aurora` references to `generate`/`sdd`.
- **Genesis → cclab-sdd/**: Moved 39 workflow specs + `run-change/` into `cclab-sdd/`, renamed all `genesis_*` tool references to `sdd_*`.
- **Legacy Cleanup**: Deleted 13+ deprecated files from `_legacy/` and 6 transitional merge specs.
- **File Renames**: `aurora-codegen-system.md` → `codegen-system.md`, `genesis-codegen-orchestration.md` → `codegen-orchestration.md`, `genesis-implement-integration.md` → `implement-integration.md`, `genesis-spec-generation.md` → `spec-generation.md`.
- Related specs: cclab-sdd/README.md

## 2026-02-15: Pulsar Data Science Expansion (pulsar-ds-expansion)
Expanded the Pulsar ecosystem with core data science features including DataFrames, Series, JSON I/O, Time Series analysis, Machine Learning, and SVG visualization.
- **Core DataFrame & Series**: Formalized the design of the foundational two-dimensional and one-dimensional data structures for high-performance numerical analysis.
- **JSON I/O (io-extra)**: Added support for reading and writing DataFrames to JSON with nested flattening.
- **Time Series Analysis (ts)**: Implemented ARIMA and smoothing algorithms with Rayon-parallelized estimation.
- **Machine Learning (ml)**: Added regression (linear/logistic), clustering (K-Means), and PCA algorithms.
- **SVG Visualization (viz)**: Implemented pure SVG 1.1 chart generation for DataFrames.
- **Python Bindings (python)**: Standardized PyO3 integration patterns across all new features.
- Related specs: cclab-pulsar/pulsar-dataframe-design.md, cclab-pulsar/pulsar-series-design.md, cclab-pulsar/pulsar-io-json.md, cclab-pulsar/pulsar-timeseries-design.md, cclab-pulsar/pulsar-ml-design.md, cclab-pulsar/pulsar-viz-design.md, cclab-pulsar/pulsar-python-bindings.md

## 2026-02-15: Mamba Feature Bundle #305-#316 (mamba-features-305-316)
Implement 12 GitHub issues (#305-#316) covering major Mamba compiler and runtime features.
- **LLVM Backend** (#305): AOT compilation via LLVM IR generation with CodegenBackend trait.
- **Import System** (#306): Multi-file module resolution, caching, and circular import handling.
- **OOP Model** (#307): C3 MRO, super(), dunder methods, and attribute access model.
- **Codegen Logic** (#308, #309): Comprehension lowering, generator codegen, and pattern matching.
- **Stdlib** (#310): Core sys, os, math, and json modules.
- **Iteration Protocol** (#311): For-loop __iter__/__next__ protocol with built-in iterators.
- **String Runtime** (#312): f-string interpolation (PEP 701) and string operations.
- **Async Runtime** (#313): Coroutine scheduling, cooperative sleep, gather, GIL management, and Orbit bridge.
- **Type System** (#314): PEP 695 generics and protocol types (structural subtyping).
- **GC Runtime** (#315): Cycle-detecting mark-sweep garbage collector.
- **REPL** (#316): Interactive mode with incremental JIT compilation and persistent state.
- Related specs: cclab-mamba/mamba-llvm-backend.md, cclab-mamba/mamba-import-system.md, cclab-mamba/mamba-oop-model.md, cclab-mamba/mamba-codegen-logic.md, cclab-mamba/mamba-stdlib-core.md, cclab-mamba/mamba-iteration-protocol.md, cclab-mamba/mamba-string-runtime.md, cclab-mamba/mamba-async-runtime.md, cclab-mamba/mamba-type-system.md, cclab-mamba/mamba-gc-runtime.md, cclab-mamba/mamba-repl-tool.md

## 2026-02-12: Verdict Enum Unification (verdict-unification)
Standardize all SDD review verdicts to use APPROVED/REVIEWED/REJECTED for system-wide consistency and bug reduction.
- **Unified Verdict Names**: Updated all spec files and Rust enums to use the new standard, replacing ad-hoc names like PASS, NEEDS_FIX, and NEEDS_REVISION.
- **Spec Correction**: Fixed contradictions in review-spec.md and unified routing logic across the workflow.
- **System-wide Alignment**: Updated MCP tool schemas, prompt templates, and state manager to recognize the unified verdict names with backwards compatibility.
- Related specs: cclab-genesis/verdict-unification.md

## 2026-02-06: Rust Symbol Analysis (lens-rust-symbols)
Extract semantic symbols from Rust source code within the Lens analysis engine.
- **Rust Symbol Extraction**: Defined the algorithm for extracting functions, structs, traits, impls, and constants from Rust files using tree-sitter.
- **Type Parsing**: Added support for parsing complex Rust type signatures into the unified TypeInfo structure.
- **Doc Comment Support**: Implemented extraction of both outer (///) and inner (//! ) documentation comments.
- Related specs: cclab-lens/rust-symbol-analysis.md

## 2026-02-05: Orbit Pipes and Zero-Copy I/O (orbit-pipes-zerocopy)
Add cross-platform pipes, zero-copy I/O, protocol lifecycle, and comprehensive testing to cclab-orbit.
- **Cross-Platform Pipes**: Unified abstraction for Unix FIFO and Windows Named Pipes.
- **Zero-Copy I/O**: High-performance data transfer using splice/sendfile and TransmitFile.
- **Protocol Lifecycle**: Async lifecycle hooks (connection_made, data_received, etc.) with timeout handling.
- **Buffer Pool**: Reusable I/O buffers to reduce allocation overhead.
- **Comprehensive Testing**: Integration and stress test suites for event loop stability.
- Related specs: cclab-orbit/unix-pipes.md, cclab-orbit/windows-pipes.md, cclab-orbit/pipe-abstraction.md, cclab-orbit/protocol-lifecycle.md, cclab-orbit/buffer-pool.md, cclab-orbit/zero-copy-io.md, cclab-orbit/integration-tests.md, cclab-orbit/benchmarks.md, cclab-orbit/stress-tests.md
- Replaces: cclab-orbit/orbit-named-pipes.md, cclab-orbit/orbit-zero-copy-apis.md

## 2026-02-05: Orbit Core Performance (orbit-core-perf)
Implement 5 core performance optimizations for cclab-orbit event loop.
- **Async-native Coroutine Polling**: Replaces 10ms sleep loop with waker-driven mechanism and GIL release.
- **Lock-free MPSC Task Queue**: Uses crossbeam channels for scalable, contention-free task dispatch.
- **Hashed Hierarchical Timer Wheel**: Implements O(1) timer operations with 1ms resolution and 4-level hierarchy.
- **Adaptive GIL Batching**: Dynamically calculates GIL batch size based on queue depth to optimize latency and throughput.
- **io_uring Backend for Linux**: High-performance I/O with zero-copy support and epoll fallback.
- Related specs: cclab-orbit/gil-waker-polling.md, cclab-orbit/mpsc-task-queue.md, cclab-orbit/hashed-timer-wheel.md, cclab-orbit/adaptive-gil-batching.md, cclab-orbit/io-uring-backend.md

## 2026-02-05: Plan Change MCP Tool (cli-to-mcp)
Migrate plan-change orchestration to a state-aware MCP tool (sdd_plan_change) and keep CLI as a thin wrapper.
- **Plan Change MCP Tool**: Dedicated tool that inspects STATE.yaml and artifacts to return next action and instructions.
- **RPC-API Specification**: Defined the OpenRPC 1.3 interface for the new planning orchestration tool.
- **Workflow Migration**: Consolidated and updated the planning logic from the CLI into a centralized service.
- Related specs: cclab-genesis/plan-change.md

## 2026-02-05: Grid DB Persistence Refactor (178-grid-db-refactor)
Refactor grid-db persistence with shared WAL crate, Morton cell storage, and yrs snapshot CRDT payloads.
- **Shared WAL Crate**: Extracted WAL logic into a new \`cclab-wal\` crate for consistent durability across storage engines.
- **Morton Cell Storage**: Implemented \`CellStore\` with Morton (Z-order) encoding for efficient rectangular range queries.
- **yrs CRDT Integration**: Refactored CRDT module to use \`yrs\` updates/snapshots for better collaboration compatibility.
- Related specs: cclab-grid-db/grid-db-architecture.md, cclab-genesis/grid-db-architecture.md

## 2026-02-04: Nebula Phase 2 (nebula-phase2)
Finalize the migration of core aggregation pipeline and batched link fetching logic to Rust with thin PyO3 wrappers.
- **Aggregation Delegation**: Single-value aggregations (avg/sum/min/max/count) now use Rust-native pipeline construction and execution.
- **Batched Link Fetching**: Forward link resolution is fully offloaded to Rust, with Python handling only metadata preparation and result hydration.
- **LinkField Metadata Contract**: Robust metadata passing between Python and Rust to support list Link and optionality.
- **Error Parity**: Preserved exception types and messages across the Rust migration boundary.
- Related specs: nebula/query-builder.md, nebula/aggregation.md, cclab-nebula/link-fetching.md

## 2026-02-03: Nebula Thin Wrapper Migration (nebula-thin-wrapper)
Migrate cclab.nebula Python logic to Rust thin wrapper architecture with full parity for performance and type safety.
- **Rust-Backed Query Builder**: Delegates read and write operations (update, delete, upsert) to Rust PyQueryBuilder.
- **Aggregation Migration**: Offloads aggregation execution and security validation to Rust.
- **Batched Link Fetching**: Full integration of the Rust-native link fetching engine.
- **Bulk Write Operations**: High-performance bulk writes exposed via PyO3.
- **State Management**: Copy-on-Write change tracking implemented in Rust for reduced overhead.
- Related specs: cclab-nebula/query-builder.md, cclab-nebula/aggregation.md, cclab-nebula/link-fetching.md, cclab-nebula/bulk-write.md, cclab-nebula/state-management.md

## 2026-02-03: Platform Sync (platform-sync)
Add platform sync service to sync SDD change artifacts to GitHub issues with automatic issue tracking.
- **GitHub Issue Sync**: Create/update GitHub issues from proposal and spec files via API or gh CLI fallback.
- **Auto Issue Tracking**: Issue numbers stored in frontmatter (\`github_issue\` field) and auto-updated after sync.
- **Scope Labels**: Auto-detection of \`crate:xxx\` labels from \`affected_code\` paths.
- **Spec Child Issues**: Specs synced as separate issues linked via tasklist in parent issue.
- **Security Hardening**: Token redaction in error messages, path traversal protection, improved .env parsing.
- **Config Support**: Platform config from \`.aw/config.toml\` (preferred) or \`.aw/config.yaml\`.

## 2026-02-03: SDD & Aurora Bug Fixes (genesis-aurora-fixes)
Fix Mermaid+ generator formatting, remove Bash permission from impl-change, and align specs to Mermaid+-only guidance.
- **Mermaid+ Format Fix**: Corrected Aurora generators to output frontmatter inside the mermaid code block for proper platform rendering.
- **Security Hardening**: Removed Bash from allowed tools in impl-change workflow to minimize risk.
- **Spec Alignment**: Removed outdated XState references from planning and format specs.
- **Robust Planning**: Enhanced plan-change to handle no-specs cases gracefully.
- Related specs: cclab-genesis/bug-fixes.md, cclab-genesis/plan-change.md, cclab-aurora/mermaid-plus-format.md, cclab-aurora/mermaid-plus-conversion.md

## 2026-02-02: Aurora Code Generation (aurora-codegen)
Implement a template-based code generation system in cclab-aurora using Tera, supporting FastAPI, Express, and Axum, with integration into cclab-probe for testing.
- **Aurora Code Generation System Architecture**: Defines the core architecture for framework-agnostic code generation.
- **JSON Schema Core Implementation**: Strongly-typed Rust structure for parsing and manipulating JSON Schema.
- **Spec Completeness Validator**: Logic for ensuring specs are generation-ready.
- **Tera Template Engine**: Core rendering service for generating code from templates.
- **Framework Generators**: Pluggable generators for FastAPI, Express, and Axum.
- **Test Generation**: Integration with cclab-probe for automated test suite generation.
- **Task Generator Nested Spec Deduplication**: Fixes task generator issues with duplicate tasks when specs exist in multiple directories.
- Related specs: cclab-aurora/aurora-codegen-system.md, cclab-aurora/json-schema-core.md, cclab-aurora/spec-validator.md, cclab-aurora/template-engine.md, cclab-aurora/generator-fastapi.md, cclab-aurora/generator-express.md, cclab-aurora/generator-axum.md, cclab-aurora/test-generation.md, cclab-genesis/task-generator-dedup.md

## 2026-02-02: Main Spec Awareness (main-spec-awareness)
Integrated main spec awareness into the SDD planning workflow to enable agents to understand the existing system state.
- **Main Spec Tools**: Added MCP tools \`list_main_specs\` and \`read_main_spec\` for accessing the \`.aw/tech-design/\` directory.
- **Spec Metadata**: Updated \`SpecFrontmatter\` with \`spec_group\`, \`main_spec_ref\`, and \`merge_strategy\` fields for better spec lifecycle management.
- Related specs: cclab-genesis/main-spec-integration.md

## 2026-02-02: Titan-Shield Unification (titan-shield-unify)
Unify titan with shield for validation by removing duplicated code and using shield as the source of truth.
- **Add Shield Dependency**: Added \`cclab-shield\` as a workspace dependency in \`crates/cclab-titan/Cargo.toml\`.
- **Remove Duplicated Code**: Removed \`crates/cclab-titan/src/pydantic_validation.rs\` to eliminate code duplication (approx 750 lines).
- **Re-export Shield Types**: Updated \`crates/cclab-titan/src/lib.rs\` to re-export validation types from \`cclab-shield\`.
- **Breaking Change**: Updated \`ValidationErrors::into_result\` signature for ecosystem alignment.
- Related specs: titan-shield-integration.md

## 2026-02-02: Nucleus PyO3 Migration (nucleus-pyo3-migration)
Migrate pyo3 bindings from cclab to individual crates and deprecate nucleus.
- **Shared Core Bindings**: Moved shared Python integration logic (BSON, types, error handling) to cclab-core.
- **Individual Crate Bindings**: Implemented pyo3_bindings in Nebula, Photon, Ion, Meteor, Nova, Probe, and Orbit.
- **Nucleus Deprecation**: Removed cclab crate from the workspace.
- **SDD Fixes**: Enhanced SDD to support recursive spec scanning and target_crate subdirectories.
- Related specs: cclab/architecture.md

## 2026-01-31: Nova Async Clarification (nova-async-clarification)
Add async clarification workflow to AnalystAgent for interactive requirements gathering via issue comments.
- **Platform Commenting**: Added \`post_comment\` to PlatformIntegration trait with implementations for GitHub, GitLab, and Jira.
- **Clarification Tools**: Added \`PostCommentTool\` for posting checkbox-style questions and \`parse_clarification_response\` for parsing user responses.
- **Session Pause/Resume**: Implemented \`pause_for_clarification\`, \`resume_with_response\` in AnalystAgent with full LLM message history preservation.
- **Storage Extensions**: Added \`messages\` and \`pending_clarification\` fields to SessionState for context preservation during async workflows.
- Related specs: analyst-agent-async.md, clarification-tools.md, platform-commenting.md

## 2026-01-31: Nova Analyst Agent (nova-analyst-agent)
Add AnalystAgent to cclab-nova for requirements analysis with composable integrations (GitHub, Jira) and session persistence.
- **AnalystAgent**: Specialized agent implementation for research and requirements gathering.
- **Analysis Tools**: New tools for information gathering (WebSearch, WebFetch), user interaction (AskUser), and note taking.
- **Platform Integrations**: Composable connectors for external project management tools (GitHub, GitLab, Jira).
- **Storage Backend**: Pluggable storage system for persisting analysis sessions (Memory, File-based).
- Related specs: analyst-agent.md, analysis-tools.md, storage-backend.md, platform-integrations.md

## 2026-01-31: Nova Streaming (nova-streaming)
Fix Claude provider and add full streaming support for cclab-nova-llm with Gemini provider.
- **HttpClient Streaming**: Added \`execute_stream\` and \`execute_builder_stream\` to cclab-photon for async byte streaming.
- **Unified StreamResponse**: Introduced \`StreamResponse\` type alias in cclab-nova-llm for consistent streaming interface.
- **Claude Provider Fixes**: Fixed type mismatches in ClaudeProvider (ToolCall.arguments, model field check).
- **Claude Streaming**: Implemented streaming support returning unified StreamResponse.
- **Gemini Provider**: Full GeminiProvider implementation with completion and chunked JSON streaming.
- **OpenAI Alignment**: Updated OpenAIProvider to use StreamResponse type.
- Related specs: cclab-nova-llm-streaming.md

## 2026-02-01: Batched Link Fetching in Rust (nebula-rust-link-fetch)
Moved the link fetching pipeline to Rust to significantly improve performance and support deep recursive fetching.
- **Rust-Native Link Processing**: Implemented foundational link types (\`LinkField\`, \`LinkRef\`) and reference extraction logic in \`cclab-nebula\` for zero-overhead processing.
- **High-Performance Batch Engine**: Added a new \`fetch_links_batched\` async engine in Rust that groups references by collection and performs parallel queries, eliminating the N+1 query problem.
- **Recursive Depth Support**: The new Rust engine supports recursive fetching of nested links up to 5 levels deep with optimized reference tracking to avoid redundant queries.
- **Low-Latency Python Bindings**: Exposed the batched fetching engine via PyO3, enabling high-speed link resolution for Python documents with minimal GIL contention.
- Related specs: link-fetch-types.md, link-fetch-pyo3.md

## 2026-01-30: Initialize Pulsar Array Core (pulsar-array-core)
Implemented foundational N-dimensional array crate for the Pulsar ecosystem, providing a pure-Rust, zero-dependency alternative to NumPy.
- **N-Dimensional Storage**: Supports arrays of arbitrary dimensions (rank) stored in a contiguous memory block with shape and stride metadata.
- **Flexible DType System**: Core numeric types (f32, f64, i32, i64) and boolean types using a unified DType system.
- **Broadcasting Support**: Automatic expansion of array shapes during operations between arrays of different but compatible dimensions.
- **Slicing and Indexing**: Efficient mechanisms for viewing and manipulating subsets of data without unnecessary copying.
- **Basic Arithmetic Operations**: Element-wise mathematical operations (+, -, *, /) implemented via standard Rust traits.
- Related specs: cclab-pulsar-array-core/pulsar-array-core-design.md

## 2026-01-29: Improve Quasar Maturity (improve-quasar-maturity)
Upgraded cclab-quasar to 95% maturity by implementing modern API framework features, automated dependency resolution, and enhanced testing utilities.
- **Automated Dependency Injection**: Implemented graph-resolving DI for route handlers using FastAPI-style 'Depends' markers.
- **Interactive Documentation**: Integrated built-in support for Swagger UI and ReDoc to serve API specifications directly from the framework.
- **Lifespan Management**: Added reliable startup and shutdown event hooks integrated into the server run loop.
- **In-Process TestClient**: Created a comprehensive TestClient for high-speed integration testing without TCP binding.
- **Robust Test Coverage**: Expanded the test suite to cover complex middleware chains, WebSocket disconnections, and SSE keep-alive heartbeats.
- Related specs: quasar-maturity-upgrade.md, quasar-di.md, quasar-docs.md, quasar-lifespan.md, quasar-test-client.md, quasar-test-expansion.md

## 2026-01-28: Improve Titan Maturity (improve-titan-maturity)
Upgrade cclab-titan maturity to 95% with dialect abstraction, robust transaction management, and enhanced validation capabilities.
- **Multi-Dialect Support**: Introduced a new \`Dialect\` trait and implementations for PostgreSQL, SQLite, and MySQL using sqlx.
- **Transaction Reliability**: Enhanced transaction management with support for configurable isolation levels, access modes, and savepoints.
- **Pydantic-style Validation & Computed Fields**: Integrated a robust validation system in the Rust core with support for model-level validators and computed fields.
- **Enhanced Connection Resilience**: Improved connection pooling logic with configurable retry strategies and exponential backoff.
- **Testing and Documentation Gaps**: Closed identified gaps with cross-dialect integration tests and comprehensive architectural guides.
- Related specs: dialect-abstraction.md, session-unit-of-work.md, hook-system.md, hybrid-properties.md, test-doc-gaps.md

## 2026-01-28: Improve Nova Maturity (improve-nova-maturity)
Upgrade cclab-nova to 95% maturity as a high-performance PydanticAI/LangGraph alternative.
- **Python Bindings (PyO3)**: Implemented comprehensive PyO3 bindings for Agent, Tool, AgentContext, and Graph classes, with full asyncio integration.
- **New Graph Engine**: Added \`cclab-nova-graph\` with a high-performance DAG executor, state propagation using Copy-on-Write (Arc), and conditional branching.
- **Structured Output Validation**: Integrated \`cclab-shield\` for schema-validated LLM responses.
- **Dependency Injection**: Enhanced \`AgentContext\` to support a generic \`RunContext\` for resource injection into tools and agents.
- **Conversation Persistence**: Added support for Python (sqlx) and Redis persistence adapters for session history.
- **Unified LLM Interface**: Unified streaming, tool calling, and structured output across OpenAI, Anthropic Claude, and gateways (LiteLLM, OpenRouter).
- **Standard Toolset**: Implemented a core registry with Web Search (Brave), Calculator, and Python REPL tools.
- Related specs: cclab-nova-core.md, cclab-nova-llm.md, cclab-nova-tools.md, cclab-nova-graph.md, cclab-nova-persistence.md, cclab-nova-python.md

## 2026-01-28: Improve Grid Maturity (improve-grid-maturity)
Upgrade cclab-grid maturity to 95% with full I/O support, rich styling, and advanced formula capabilities.
- **New cclab-grid-io crate**: Added read/write support for XLSX, CSV, and ODS formats.
    - **XLSX Support**: Full read support using \`calamine\` and write support using \`rust_xlsxwriter\`.
    - **CSV Support**: Configurable delimiter and encoding support for fast data import/export.
    - **ODS Support**: Basic read/write support for OpenDocument Spreadsheet format.
- **Expanded Styling Engine**: Added support for borders, pattern fills, and workbook themes.
    - **Cell Borders**: Support for solid, dashed, dotted, and thick borders on all four sides with custom colors.
    - **Pattern Fills**: Added support for various pattern types including diagonal stripes and gray shades with foreground/background colors.
    - **Workbook Themes**: Implemented a theme system for consistent color palettes across the workbook.
- **Advanced Formula Support**: Implemented INDEX, improved VLOOKUP/MATCH with wildcard support, and added Array Formulas (CSE).
    - **INDEX Function**: Added support for 1D and 2D array lookups.
    - **Wildcard Matching**: Added \`*\` and \`?\` wildcard support to \`VLOOKUP\` and \`MATCH\` functions.
    - **Dynamic Spilling**: Formulas can now return ranges that automatically "spill" into adjacent empty cells.
    - **Spill Collision Detection**: Implemented \`#SPILL!\` error handling when data blocks expansion.
- **Circular Dependency Detection**: Implemented robust detection using DFS in the formula engine, returning \`#REF!\` for affected formulas.
- **Performance Optimizations**: Optimized for 100k+ row datasets with responsive recalculation (<500ms for 1,000 cells).
- **Benchmarking Suite**: Added a new suite for tracking performance of core operations (I/O, evaluation, rendering).
- Related specs: grid-io-spec.md, grid-styling-spec.md, grid-formula-array-spec.md, grid-formula-functions-spec.md, grid-performance-spec.md


## 2026-01-27: Improve Gemini Spec Generation (improve-spec-generation)
Improved the quality and reliability of automated spec generation by mandating formal specification languages and strengthening automated validation.
- **Formal Language Mandate**: Updated spec creation and revision prompts to include comprehensive, copy-pasteable examples for OpenAPI 3.1, AsyncAPI 2.6, OpenRPC 1.3, and Serverless Workflow 0.8.
- **Centralized Rules**: Refactored the orchestrator and validation tools to use a centralized source of truth (SpecType enum) for required diagrams and API specifications, ensuring consistency between AI prompts and automated checks.
- **Strict Validation**: Enhanced validate_spec_completeness and SemanticValidator to strictly enforce the presence of machine-readable specifications and semantic metadata in diagrams.
- **Improved Prompting**: Centralized spec-type specific guidance in the orchestrator to ensure AI agents receive clear, deterministic instructions based on the specification category.
- Related specs: spec-generation-improvement.md

## 2026-01-24: Expand SDD Viewer (genesis-viewer)
Expanded the SDD Viewer from a single-change viewer into a comprehensive project-level browser. Added project-level routing and tree-based navigation. Support for hierarchical directory scanning and file preview was implemented, while full LaTeX/KaTeX and interactive table sorting were deferred to future enhancements.
- Related specs: plan-viewer.md, genesis-viewer-expansion.md

## 2026-01-17: Add Plan Viewer UI (plan-viewer-ui)
Added a standalone UI window viewer for \`genesis\` plans using \`wry\`. This viewer provides a rich, interactive interface for reviewing proposals and challenges, featuring native Mermaid diagram rendering, STATE.yaml rendering, and support for human annotations.
- Related specs: plan-viewer.md, annotations.md

## 2026-01-17: Enhance Fillback Process (improve-fillback-2)
Added \`fillback-enhancement.md\` to specify the enhanced fillback process, transitioning from simple file-scanning to AST-based analysis and interactive clarification.
- Related specs: fillback-enhancement.md

## 2026-01-16: Add dedicated archived command (test-retry)
Added a dedicated \`genesis archived\` CLI command to improve discoverability of project history and provide a detailed view of completed changes. This allowed users to browse past work with richer context, including dates and extracted summaries.
- Related specs: archived-command.md
