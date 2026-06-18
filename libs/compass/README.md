# Compass

## Brief

Compass is the cclab code-intelligence library for navigating, checking,
searching, refactoring, generating, and incrementally watching codebases.

It exposes Rust APIs for tree-sitter parsing, language-specific linting,
semantic/type analysis, code search, refactoring operations, spec parsing,
code generation, and the Argus daemon/watch stack. The library has broad unit
coverage, but the configured full gate is currently blocked by a doctest that
still imports the retired `sdd::server::incremental` path.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Codebase Check And Lint Pipeline | - | implemented | failing | smoke | not_ready | unit coverage passes inside the full run; doctest failure blocks the configured project gate |
| Semantic Navigation Search And Refactoring | - | implemented | failing | smoke | not_ready | symbol, type, search, PDG, and refactoring tests pass inside the full run; doctest failure blocks the configured project gate |
| Spec Parsing And Code Generation | - | implemented | failing | smoke | not_ready | parser/generator tests pass inside the full run; doctest failure blocks the configured project gate |
| Daemon Watch And Incremental Analysis | - | implemented | failing | smoke | not_ready | daemon/watch/incremental tests pass inside the full run; stale doctest import blocks the configured project gate |

### Codebase Check And Lint Pipeline

ID: codebase-check-and-lint-pipeline
Type: AgentFirst
Surfaces: Rust API: `check_paths`, `check_paths_with_propagation`, `LintConfig`, `FileResult`, `CheckerRegistry`, `Checker`, `Diagnostic`, `Reporter`; Modules: `syntax`, `lint`, `format`, `output`
EC Dimensions: behavior: `cargo test -p cclab-compass` - configured parser, checker, diagnostic, and output smoke gate
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Compass can parse source files, dispatch language-specific checkers, return diagnostics, and emit agent-readable reports across supported code and document formats.
Gate Inventory: `cargo test -p cclab-compass`; libs/compass/src/checker.rs; libs/compass/src/lint/mod.rs; libs/compass/src/syntax/mod.rs; libs/compass/src/output/agent.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Multi-language parser and checker dispatch contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-compass`; libs/compass/src/checker.rs; libs/compass/src/lint/mod.rs |
| Agent diagnostic output contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-compass`; libs/compass/src/output/agent.rs; libs/compass/src/output/reporter.rs |

### Semantic Navigation Search And Refactoring

ID: semantic-navigation-search-and-refactoring
Type: AgentFirst
Surfaces: Rust API: `outline`, `outline_parsed`, `type_at`, `hover`, `SearchEngine`, `RefactoringRegistry`, `DeepTypeInferencer`, `PropagationPipeline`; Modules: `semantic`, `graph`, `search`, `type_inference`, `refactoring`, `outline`
EC Dimensions: behavior: `cargo test -p cclab-compass` - configured semantic, type inference, search, and refactoring smoke gate
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Compass provides agent-facing navigation primitives for symbol outlines, propagated type and hover answers, dependency graphs, semantic search, PDG-style impact analysis, and structured refactoring operations.
Gate Inventory: `cargo test -p cclab-compass`; libs/compass/src/check_pipeline.rs; libs/compass/src/search/mod.rs; libs/compass/src/refactoring/mod.rs; libs/compass/src/semantic/mod.rs; libs/compass/src/type_inference/mod.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Symbol outline and propagated type query contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-compass`; libs/compass/src/check_pipeline.rs; libs/compass/src/outline.rs |
| Semantic search and graph query contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-compass`; libs/compass/src/search/mod.rs; libs/compass/src/semantic/pdg/mod.rs |
| Structured refactoring contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-compass`; libs/compass/src/refactoring/mod.rs; libs/compass/src/type_inference/refactoring.rs |

### Spec Parsing And Code Generation

ID: spec-parsing-and-code-generation
Type: DeveloperTool
Surfaces: Rust API: `GeneratorRegistry`, `CodeGenerator`, `GenContext`, `GeneratedCode`, `TechStack`, `StateMachineValidator`, `MermaidPlusGenerator`; Modules: `spec`, `gen`
EC Dimensions: behavior: `cargo test -p cclab-compass` - configured spec parser and generator smoke gate
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Compass parses structured specifications such as JSON Schema, OpenAPI, AsyncAPI, Mermaid, and state-machine definitions, then provides generator traits and registry-backed generators for Python and Rust code targets.
Gate Inventory: `cargo test -p cclab-compass`; libs/compass/src/spec/mod.rs; libs/compass/src/gen/mod.rs; libs/compass/src/gen/registry.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Spec parser and state-machine validation contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-compass`; libs/compass/src/spec/mod.rs; libs/compass/src/spec/statemachine/mod.rs |
| Python and Rust generator registry contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-compass`; libs/compass/src/gen/mod.rs; libs/compass/src/gen/registry.rs |

### Daemon Watch And Incremental Analysis

ID: daemon-watch-and-incremental-analysis
Type: Service
Surfaces: Rust API: `ArgusDaemon`, `DaemonClient`, `DaemonConfig`, `RequestHandler`, `FileWatcher`, `WatchConfig`, `WatchEvent`, `IncrementalUpdateManager`, `DirtyFileTracker`, `DependencyGraph`, `WatchBridge`; Protocol: JSON-RPC over Unix socket
EC Dimensions: behavior: `cargo test -p cclab-compass` - configured daemon, watch, and incremental update smoke gate
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Compass can run a local Argus analysis daemon, track file changes, maintain dependency-aware dirty-file sets, bridge filesystem watcher events into incremental analysis, and serve JSON-RPC code-intelligence requests.
Gate Inventory: `cargo test -p cclab-compass`; libs/compass/src/server/mod.rs; libs/compass/src/server/incremental.rs; libs/compass/src/server/watch_bridge.rs; libs/compass/src/watch.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Argus daemon protocol and request handling contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-compass`; libs/compass/src/server/mod.rs; libs/compass/src/server/protocol.rs |
| Watch bridge and incremental dirty-file contract | epic | - | implemented | failing | smoke | `cargo test -p cclab-compass`; libs/compass/src/server/incremental.rs; libs/compass/src/server/watch_bridge.rs; libs/compass/src/watch.rs |
