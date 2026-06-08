---
id: compiler-driver
title: Compiler Driver — Pipeline Orchestration
crate: mamba
files:
  - crates/mamba/src/driver/mod.rs
  - crates/mamba/src/driver/module_graph.rs
  - crates/mamba/src/driver/config.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: bf867ab85
---

# Compiler Driver

`driver/mod.rs` (881 LOC) is the top-level orchestrator — wires
parser, lower, type-check, codegen, JIT, and runtime into a single
pipeline. `module_graph.rs` (614 LOC) tracks dependency edges between
imported modules so re-compilation only touches what changed.
`config.rs` (274 LOC) defines `MambaConfig` (driver options:
target backend, optimization level, search paths).

Three load-bearing invariants:

1. **`MambaConfig` is unified between driver and schema layers** —
   commit `1ba891cc4` (#1134) merged the previously-dual `driver/config.rs`
   and `config/schema.rs` into one canonical type. Future config
   additions go in this one place.
2. **Module graph cycles are detected and rejected at parse time** —
   import cycles raise `ImportError`; downstream compilation paths
   assume DAG ordering. The graph also drives incremental rebuild
   ordering.
3. **The driver is the only place that creates `CraneliftJitBackend`
   instances for top-level modules** — module dependencies route
   their JIT-backends through `runtime/module::MODULE_JIT_BACKENDS`
   (per `module.md`) but the entry-point backend is owned by the
   driver until execution completes.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: driver-types
types:
  Driver:        { kind: struct, label: "driver/mod.rs entry — run / build / repl" }
  ModuleGraph:   { kind: struct, label: "DAG of imports + change detection" }
  MambaConfig:   { kind: struct, label: "unified config — target / opt level / paths" }
  ParserMod:     { kind: struct, label: "from parser/mod" }
  TypeChecker:   { kind: struct, label: "from types" }
  Lower:         { kind: struct, label: "from lower (ast→hir, hir→mir)" }
  Codegen:       { kind: struct, label: "from codegen/cranelift or llvm" }
  Runtime:       { kind: struct, label: "from runtime — module + symbol registries" }
edges:
  - { from: Driver,      to: ModuleGraph, kind: owns }
  - { from: Driver,      to: MambaConfig, kind: owns }
  - { from: Driver,      to: ParserMod,   kind: references }
  - { from: Driver,      to: TypeChecker, kind: references }
  - { from: Driver,      to: Lower,       kind: references }
  - { from: Driver,      to: Codegen,     kind: references }
  - { from: Driver,      to: Runtime,     kind: references }
  - { from: ModuleGraph, to: ParserMod,   kind: references, label: "topological re-parse" }
---
classDiagram
    class Driver
    class ModuleGraph
    class MambaConfig
    class ParserMod
    class TypeChecker
    class Lower
    class Codegen
    class Runtime
    Driver --> ModuleGraph : owns
    Driver --> MambaConfig : owns
    Driver --> ParserMod : refs
    Driver --> TypeChecker : refs
    Driver --> Lower : refs
    Driver --> Codegen : refs
    Driver --> Runtime : refs
    ModuleGraph --> ParserMod : topological
```

## Driver shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "driver-types"
$defs:
  MambaConfig:
    type: object
    x-rust-type: MambaConfig
    properties:
      target:        { type: string, description: "e.g. aarch64-apple-darwin / x86_64-unknown-linux-gnu" }
      backend:       { type: string, enum: [cranelift, llvm] }
      opt_level:     { type: string, enum: [O0, O1, O2, O3] }
      search_paths:  { type: array, items: { type: string } }
      script_dir:
        oneOf:
          - { type: "null" }
          - { type: string }
      output_path:
        oneOf:
          - { type: "null" }
          - { type: string }
      mode:          { type: string, enum: [run, build, repl, test] }
    required: [target, backend, opt_level, search_paths, script_dir, output_path, mode]
  ModuleNode:
    type: object
    properties:
      name:          { type: string }
      file:          { type: string }
      imports:       { type: array, items: { type: string } }
      hash:          { type: string, description: "content hash for change detection" }
      compiled:      { type: boolean }
    required: [name, file, imports, hash, compiled]
```

## Run / build dispatch logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: driver-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "Driver::main(MambaConfig)" }
  is_repl:      { kind: decision, label: "mode == repl?" }
  start_repl:   { kind: terminal, label: "Driver::repl() — see repl.md" }
  is_build:     { kind: decision, label: "mode == build?" }
  build_aot:   { kind: process,  label: "compile_aot all reachable modules; link object files" }
  is_run:       { kind: decision, label: "mode == run?" }
  build_dag:   { kind: process,  label: "ModuleGraph::resolve(entry) — parse imports topologically" }
  per_module:  { kind: process,  label: "for each module in topo order: parse + check + lower + codegen" }
  run_entry:   { kind: process,  label: "transmute entry fn pointer; call from main thread; capture stdout" }
  cleanup:     { kind: process,  label: "cleanup_all_runtime_state on exit (mandatory aarch64)" }
  done:         { kind: terminal, label: "exit code" }
edges:
  - { from: enter,      to: is_repl }
  - { from: is_repl,    to: start_repl, label: "yes" }
  - { from: is_repl,    to: is_build,   label: "no" }
  - { from: is_build,   to: build_aot,  label: "yes" }
  - { from: is_build,   to: is_run,     label: "no" }
  - { from: is_run,     to: build_dag,  label: "yes" }
  - { from: build_dag,  to: per_module }
  - { from: per_module, to: run_entry }
  - { from: run_entry,  to: cleanup }
  - { from: cleanup,    to: done }
  - { from: build_aot,  to: done }
---
flowchart TD
    enter([Driver main]) --> is_repl{repl?}
    is_repl -->|yes| start_repl([repl])
    is_repl -->|no| is_build{build?}
    is_build -->|yes| build_aot[AOT all + link]
    is_build -->|no| is_run{run?}
    is_run -->|yes| build_dag[ModuleGraph resolve]
    build_dag --> per_module[topo per-module pipeline]
    per_module --> run_entry[transmute + call]
    run_entry --> cleanup[cleanup_all_runtime_state]
    cleanup --> done([exit code])
    build_aot --> done
```

## Run interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: driver-run-flow
actors:
  - { id: User,    kind: actor }
  - { id: Driver,  kind: system }
  - { id: Graph,   kind: system, label: "ModuleGraph" }
  - { id: Pipeline, kind: system, label: "parser through check through lower through codegen" }
  - { id: Runtime, kind: system }
messages:
  - { from: User,    to: Driver,   name: "mamba run script.py" }
  - { from: Driver,  to: Graph,    name: "resolve(script.py)" }
  - { from: Graph,   to: Graph,    name: "parse imports recursively; topo sort" }
  - { from: Graph,   to: Driver,   name: "ordered list of modules" }
  - { from: Driver,  to: Pipeline, name: "for each module: parse + check + lower_module + lower_hir_to_mir + codegen" }
  - { from: Pipeline, to: Runtime, name: "register module via mb_module_register" }
  - { from: Pipeline, to: Driver,  name: "JIT entry fn pointer for entry module" }
  - { from: Driver,  to: Runtime,  name: "cleanup_all_runtime_state" }
  - { from: Driver,  to: User,     name: "exit code" }
---
sequenceDiagram
    actor User
    participant Driver
    participant Graph
    participant Pipeline
    participant Runtime
    User->>Driver: mamba run
    Driver->>Graph: resolve
    Graph-->>Driver: topo ordered
    Driver->>Pipeline: per-module pipeline
    Pipeline->>Runtime: mb_module_register
    Pipeline-->>Driver: entry fn ptr
    Driver->>Runtime: cleanup
    Driver-->>User: exit code
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->
```yaml
scenarios:
  - id: run-hello
    given: hello.py prints hello
    when: mamba run hello.py is executed
    then: the driver runs the pipeline and returns stdout hello
  - id: build-executable
    given: hello.py is a valid entry module
    when: mamba build hello.py -o hello is executed
    then: the driver emits an executable at the requested output path
  - id: module-topology
    given: pkg/main.py imports submodules
    when: mamba run pkg/main.py is executed
    then: ModuleGraph resolves topological order and compiles dependencies before main
  - id: circular-imports
    given: circular_imports.py contains a module import cycle
    when: the driver resolves the module graph
    then: it rejects the graph with an ImportError before downstream compilation
```

## Tests
<!-- type: test-plan lang: mermaid -->
```mermaid
---
id: compiler-driver-test-plan
title: Compiler Driver Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test runtime_tests --release -- {name} --test-threads=1"]
    Runner --> RunHello["test_driver_run_hello"]
    Runner --> BuildAot["test_driver_build_emits_executable"]
    Runner --> ModuleTopo["test_module_graph_topological_order"]
    Runner --> Circular["test_circular_imports_rejected"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/driver/mod.rs
    action: modify
    impl_mode: hand-written
    description: "Driver entry — run / build / repl / test dispatch; pipeline orchestration. Hand-written."
  - file: crates/mamba/src/driver/module_graph.rs
    action: modify
    impl_mode: hand-written
    description: "Import-graph DAG; topological ordering; cycle detection; content-hash change tracking. Hand-written."
  - file: crates/mamba/src/driver/config.rs
    action: modify
    impl_mode: hand-written
    description: "MambaConfig — unified per #1134 commit 1ba891cc4. Hand-written."
```
