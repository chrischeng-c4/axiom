# Compass

## Brief

Compass is the cclab code-intelligence library for navigating, checking,
searching, refactoring, generating, and incrementally watching codebases.

It exposes Rust APIs for tree-sitter parsing, language-specific linting,
semantic/type analysis, code search, refactoring operations, spec parsing,
code generation, and the Argus daemon/watch stack. The configured smoke gate
covers the library unit suite plus doctests; production readiness still depends
on semantic TD and traceability closure.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Codebase Check And Lint Pipeline | - | parser, checker, diagnostic, and output smoke gate passes |
| Semantic Navigation Search And Refactoring | - | symbol, type, search, PDG, and refactoring smoke gate passes |
| Spec Parsing And Code Generation | - | parser/generator smoke gate passes |
| Daemon Watch And Incremental Analysis | - | daemon, watch, and incremental analysis smoke gate passes |

### Codebase Check And Lint Pipeline

Compass can parse source files, dispatch language-specific checkers, return
diagnostics, and emit agent-readable reports across supported code and document
formats.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `check_paths`, `check_paths_with_propagation`,
  `LintConfig`, `FileResult`, `CheckerRegistry`, `Checker`, `Diagnostic`,
  `Reporter`; Modules: `syntax`, `lint`, `format`, `output`
- Gate — behavior: `cargo test -p cclab-compass` - configured parser, checker,
  diagnostic, and output smoke gate
- Gate: `cargo test -p cclab-compass`
- Source: `libs/compass/src/checker.rs`, `libs/compass/src/lint/mod.rs`,
  `libs/compass/src/syntax/mod.rs`, `libs/compass/src/output/agent.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Multi-language parser and checker dispatch contract | epic | - | `cargo test -p cclab-compass`; libs/compass/src/checker.rs; libs/compass/src/lint/mod.rs |
| Agent diagnostic output contract | epic | - | `cargo test -p cclab-compass`; libs/compass/src/output/agent.rs; libs/compass/src/output/reporter.rs |

### Semantic Navigation Search And Refactoring

Compass provides agent-facing navigation primitives for symbol outlines,
propagated type and hover answers, dependency graphs, semantic search,
PDG-style impact analysis, and structured refactoring operations.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `outline`, `outline_parsed`, `type_at`, `hover`,
  `SearchEngine`, `RefactoringRegistry`, `DeepTypeInferencer`,
  `PropagationPipeline`; Modules: `semantic`, `graph`, `search`,
  `type_inference`, `refactoring`, `outline`
- Gate — behavior: `cargo test -p cclab-compass` - configured semantic, type
  inference, search, and refactoring smoke gate
- Gate: `cargo test -p cclab-compass`
- Source: `libs/compass/src/check_pipeline.rs`,
  `libs/compass/src/search/mod.rs`, `libs/compass/src/refactoring/mod.rs`,
  `libs/compass/src/semantic/mod.rs`, `libs/compass/src/type_inference/mod.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Symbol outline and propagated type query contract | epic | - | `cargo test -p cclab-compass`; libs/compass/src/check_pipeline.rs; libs/compass/src/outline.rs |
| Semantic search and graph query contract | epic | - | `cargo test -p cclab-compass`; libs/compass/src/search/mod.rs; libs/compass/src/semantic/pdg/mod.rs |
| Structured refactoring contract | epic | - | `cargo test -p cclab-compass`; libs/compass/src/refactoring/mod.rs; libs/compass/src/type_inference/refactoring.rs |

### Spec Parsing And Code Generation

Compass parses structured specifications such as JSON Schema, OpenAPI,
AsyncAPI, Mermaid, and state-machine definitions, then provides generator
traits and registry-backed generators for Python and Rust code targets.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `GeneratorRegistry`, `CodeGenerator`, `GenContext`,
  `GeneratedCode`, `TechStack`, `StateMachineValidator`,
  `MermaidPlusGenerator`; Modules: `spec`, `gen`
- Gate — behavior: `cargo test -p cclab-compass` - configured spec parser and
  generator smoke gate
- Gate: `cargo test -p cclab-compass`
- Source: `libs/compass/src/spec/mod.rs`, `libs/compass/src/gen/mod.rs`,
  `libs/compass/src/gen/registry.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Spec parser and state-machine validation contract | epic | - | `cargo test -p cclab-compass`; libs/compass/src/spec/mod.rs; libs/compass/src/spec/statemachine/mod.rs |
| Python and Rust generator registry contract | epic | - | `cargo test -p cclab-compass`; libs/compass/src/gen/mod.rs; libs/compass/src/gen/registry.rs |

### Daemon Watch And Incremental Analysis

Compass can run a local Argus analysis daemon, track file changes, maintain
dependency-aware dirty-file sets, bridge filesystem watcher events into
incremental analysis, and serve JSON-RPC code-intelligence requests.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `ArgusDaemon`, `DaemonClient`, `DaemonConfig`,
  `RequestHandler`, `FileWatcher`, `WatchConfig`, `WatchEvent`,
  `IncrementalUpdateManager`, `DirtyFileTracker`, `DependencyGraph`,
  `WatchBridge`; Protocol: JSON-RPC over Unix socket
- Gate — behavior: `cargo test -p cclab-compass` - configured daemon, watch,
  and incremental update smoke gate
- Gate: `cargo test -p cclab-compass`
- Source: `libs/compass/src/server/mod.rs`,
  `libs/compass/src/server/incremental.rs`,
  `libs/compass/src/server/watch_bridge.rs`, `libs/compass/src/watch.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Argus daemon protocol and request handling contract | epic | - | `cargo test -p cclab-compass`; libs/compass/src/server/mod.rs; libs/compass/src/server/protocol.rs |
| Watch bridge and incremental dirty-file contract | epic | - | `cargo test -p cclab-compass`; libs/compass/src/server/incremental.rs; libs/compass/src/server/watch_bridge.rs; libs/compass/src/watch.rs |
