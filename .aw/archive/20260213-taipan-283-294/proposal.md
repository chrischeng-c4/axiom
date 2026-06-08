---
id: taipan-283-294
type: proposal
version: 2
created_at: 2026-02-13T04:17:26.424317+00:00
updated_at: 2026-02-13T04:17:26.424317+00:00
iteration: 1
scope: minor
spec_plan:
  - id: taipan-core-types
    title: "Core Data Structures (String, List, Dict, Tuple)"
    depends: []
    context_refs:
    affected_code: ["crates/cclab-taipan/src/runtime/objects/", "crates/cclab-taipan/src/codegen/"]
  - id: taipan-exceptions
    title: "Exception Handling System"
    depends: []
    context_refs:
    affected_code: ["crates/cclab-taipan/src/runtime/exception.rs", "crates/cclab-taipan/src/codegen/"]
  - id: taipan-functions
    title: "Closures and Decorators"
    depends: [taipan-core-types]
    context_refs:
    affected_code: ["crates/cclab-taipan/src/runtime/function.rs", "crates/cclab-taipan/src/lowering/"]
  - id: taipan-classes
    title: "Classes, Inheritance, and Operator Overloading"
    depends: [taipan-core-types, taipan-exceptions]
    context_refs:
    affected_code: ["crates/cclab-taipan/src/runtime/class.rs", "crates/cclab-taipan/src/codegen/"]
  - id: taipan-iterators
    title: "Iterators, Generators, and Comprehensions"
    depends: [taipan-classes]
    context_refs:
    affected_code: ["crates/cclab-taipan/src/runtime/iter.rs", "crates/cclab-taipan/src/codegen/"]
  - id: taipan-modules
    title: "Module Import System"
    depends: [taipan-core-types, taipan-classes]
    context_refs:
    affected_code: ["crates/cclab-taipan/src/runtime/module.rs", "crates/cclab-taipan/src/codegen/"]
  - id: taipan-async
    title: "Async/Await with Tokio"
    depends: [taipan-classes, taipan-iterators]
    context_refs:
    affected_code: ["crates/cclab-taipan/src/runtime/async.rs", "crates/cclab-taipan/src/codegen/"]
history:
  - timestamp: 2026-02-13T04:17:26.424317+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: taipan-283-294

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((taipan-283-294))  
    Language Features
      Classes
      Exceptions
      Closures
      Decorators
      Generators
      Comprehensions
      Async/Await
    Runtime
      Data Structures
      Object Model
      Tokio Integration
      Module System
    Compiler
      Codegen
      Lowering
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  taipan_core_types["taipan-core-types"]
  taipan_exceptions["taipan-exceptions"]
  taipan_functions["taipan-functions"]
  taipan_classes["taipan-classes"]
  taipan_iterators["taipan-iterators"]
  taipan_modules["taipan-modules"]
  taipan_async["taipan-async"]

  taipan_core_types --> taipan_functions
  taipan_core_types --> taipan_classes
  taipan_exceptions --> taipan_classes
  taipan_classes --> taipan_iterators
  taipan_core_types --> taipan_modules
  taipan_classes --> taipan_modules
  taipan_classes --> taipan_async
  taipan_iterators --> taipan_async
```

## Spec Execution Order

1. **taipan-core-types** — Core Data Structures (String, List, Dict, Tuple)
   - code: crates/cclab-taipan/src/runtime/objects/, crates/cclab-taipan/src/codegen/
2. **taipan-exceptions** — Exception Handling System
   - code: crates/cclab-taipan/src/runtime/exception.rs, crates/cclab-taipan/src/codegen/
3. **taipan-classes** — Classes, Inheritance, and Operator Overloading
   - depends: taipan-core-types, taipan-exceptions
   - code: crates/cclab-taipan/src/runtime/class.rs, crates/cclab-taipan/src/codegen/
4. **taipan-functions** — Closures and Decorators
   - depends: taipan-core-types
   - code: crates/cclab-taipan/src/runtime/function.rs, crates/cclab-taipan/src/lowering/
5. **taipan-iterators** — Iterators, Generators, and Comprehensions
   - depends: taipan-classes
   - code: crates/cclab-taipan/src/runtime/iter.rs, crates/cclab-taipan/src/codegen/
6. **taipan-async** — Async/Await with Tokio
   - depends: taipan-classes, taipan-iterators
   - code: crates/cclab-taipan/src/runtime/async.rs, crates/cclab-taipan/src/codegen/
7. **taipan-modules** — Module Import System
   - depends: taipan-core-types, taipan-classes
   - code: crates/cclab-taipan/src/runtime/module.rs, crates/cclab-taipan/src/codegen/

</proposal>
