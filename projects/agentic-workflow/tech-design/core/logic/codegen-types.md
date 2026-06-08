---
id: sdd-codegen-type-system
main_spec_ref: projects/agentic-workflow/logic/codegen-types.md
merge_strategy: new
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "This codegen logic TD supports CB lifecycle generation and regenerable artifact production."
---

# Sdd Codegen Type System

## Overview
<!-- type: overview lang: markdown -->

The abstract type system enables multi-language codegen from a single TD spec. YAML frontmatter uses abstract types (`integer`, `string`, `list<T>`, `optional<T>`, etc.) that are translated to each target language by a per-language translator.

For MVP, only the Rust translator is implemented. Python/TypeScript translators share the same abstract type enum and translator trait but have deferred implementations.

Translation table (Rust MVP):

| Abstract | Modifier | Rust |
|---|---|---|
| `integer` | `int_size: 64` | `i64` |
| `integer` | `unsigned: true, int_size: 64` | `u64` |
| `integer` | `int_size: 32` | `i32` |
| `string` | — | `String` |
| `bool` | — | `bool` |
| `list<T>` | — | `Vec<T>` |
| `map<K,V>` | — | `HashMap<K,V>` |
| `optional<T>` | — | `Option<T>` |
| `ref<Name>` | — | `Name` (same module) or `crate::Name` (cross-module) |
| `bytes` | — | `Vec<u8>` |
| `any` | — | `serde_json::Value` |

Type translation lives in `projects/agentic-workflow/src/generate/types.rs`. The `AbstractType` enum is parsed from YAML `type` fields in schema frontmatter. The `RustTypeTranslator` implements `TypeTranslator<String>` for Rust output.
## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: requirements
---
requirementDiagram

requirement R1 {
  id: R1
  text: "Abstract TD field types parse into a target-neutral type enum"
  risk: low
  verifymethod: test
}

requirement R2 {
  id: R2
  text: "Rust generation translates abstract types into deterministic Rust type names"
  risk: low
  verifymethod: test
}

requirement R3 {
  id: R3
  text: "Rust generation merges per-spec overrides with default Rust config"
  risk: low
  verifymethod: test
}
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios: []
```

## Diagrams
<!-- type: doc lang: markdown -->

### Mindmap
<!-- type: mindmap lang: mermaid -->
<!-- TODO: Use Mermaid Plus mindmap (YAML frontmatter inside mermaid block).
```mermaid
---
id: mindmap
---
mindmap
  root((System))
    Component A
    Component B
```
-->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO: Use Mermaid Plus stateDiagram-v2 (YAML frontmatter inside mermaid block).
```mermaid
---
id: state-machine
initial: idle
---
stateDiagram-v2
    [*] --> idle
```
-->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO: Use Mermaid Plus sequenceDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: interaction
---
sequenceDiagram
    actor User
    User->>System: action
```
-->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO: Use Mermaid Plus flowchart (YAML frontmatter inside mermaid block).
```mermaid
---
id: logic
---
flowchart TD
    A([Start]) --> B{Decision}
```
-->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO: Use Mermaid Plus classDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: dependency
---
classDiagram
    class ComponentA
    class ComponentB
    ComponentA --> ComponentB
```
-->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO: Use Mermaid Plus erDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: db-model
---
erDiagram
    ENTITY {
        string id PK
    }
```
-->

## API Spec
<!-- type: doc lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: yaml -->
<!-- TODO: OpenRPC 1.3 as YAML. Example:
```yaml
openrpc: "1.3.2"
info:
  title: Service Name
  version: "1.0.0"
methods: []
```
-->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

### Schema
<!-- type: schema lang: yaml -->

```yaml
"$schema": "https://json-schema.org/draft/2020-12/schema"
type: object
properties:
  abstract_type:
    type: string
  rust_config:
    type: object
required: [abstract_type]
```

### Config
<!-- type: config lang: yaml -->
<!-- score-td-placeholder -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: test-plan
---
requirementDiagram

element T1 {
  type: "Test"
}

element T2 {
  type: "Test"
}

element T3 {
  type: "Test"
}

T1 - verifies -> R1
T2 - verifies -> R2
T3 - verifies -> R3
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/types.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: |
      Abstract type system for multi-language codegen.
      pub enum AbstractType { Integer { int_size: u8, unsigned: bool }, String, Bool, Bytes, Any,
        List { item: Box<AbstractType> }, Map { key: Box<AbstractType>, value: Box<AbstractType> },
        Optional { inner: Box<AbstractType> }, Ref { name: String } }
      pub trait TypeTranslator { fn translate(&self, t: &AbstractType) -> String; }
      pub struct RustTypeTranslator;
      impl TypeTranslator for RustTypeTranslator { ... }
      pub fn parse_abstract_type(yaml_str: &str) -> Result<AbstractType>
      pub struct RustConfig { pub derives: Vec<String>, pub serde_rename_strategy: String,
        pub visibility: String, pub derive_hash: bool, pub derive_copy: bool }
      impl Default for RustConfig (use project standard defaults)
      pub fn merge_overrides(&self, frontmatter: &serde_yaml::Value) -> RustConfig
  - action: annotate
    section: async-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the async-api section."

  - action: annotate
    section: cli
    impl_mode: hand-written
    description: "Traceability metadata edge for the cli section."

  - action: annotate
    section: component
    impl_mode: hand-written
    description: "Traceability metadata edge for the component section."

  - action: annotate
    section: config
    impl_mode: hand-written
    description: "Traceability metadata edge for the config section."

  - action: annotate
    section: db-model
    impl_mode: hand-written
    description: "Traceability metadata edge for the db-model section."

  - action: annotate
    section: dependency
    impl_mode: hand-written
    description: "Traceability metadata edge for the dependency section."

  - action: annotate
    section: design-token
    impl_mode: hand-written
    description: "Traceability metadata edge for the design-token section."

  - action: annotate
    section: interaction
    impl_mode: hand-written
    description: "Traceability metadata edge for the interaction section."

  - action: annotate
    section: logic
    impl_mode: hand-written
    description: "Traceability metadata edge for the logic section."

  - action: annotate
    section: mindmap
    impl_mode: hand-written
    description: "Traceability metadata edge for the mindmap section."

  - action: annotate
    section: requirements
    impl_mode: hand-written
    description: "Traceability metadata edge for the requirements section."

  - action: annotate
    section: rest-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rest-api section."

  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

  - action: annotate
    section: state-machine
    impl_mode: hand-written
    description: "Traceability metadata edge for the state-machine section."

  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

  - action: annotate
    section: wireframe
    impl_mode: hand-written
    description: "Traceability metadata edge for the wireframe section."

```
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
wireframes: []
```

## Component
<!-- type: component lang: yaml -->

```yaml
components: []
```

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
tokens: []
```

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->


## Schema
<!-- type: schema lang: yaml -->

```yaml
"$schema": "https://json-schema.org/draft/2020-12/schema"
title: AbstractTypeSpec
description: Abstract type specification used in structural diagram frontmatter fields
type: object
properties:
  type:
    type: string
    description: Abstract type name
    enum:
      - integer
      - string
      - bool
      - bytes
      - any
      - "list<T>"
      - "map<K,V>"
      - "optional<T>"
      - "ref<Name>"
  int_size:
    type: integer
    description: Bit width for integer types (8, 16, 32, 64)
    enum: [8, 16, 32, 64]
    default: 64
  unsigned:
    type: boolean
    description: Whether integer type is unsigned
    default: false
  item_type:
    type: string
    description: Element type for list<T> and key type for map<K,V>
  value_type:
    type: string
    description: Value type for map<K,V>
  ref_name:
    type: string
    description: Referenced type name for ref<Name>

---
title: RustConfig
description: Rust codegen configuration (from global config.toml + per-spec x-rust overrides)
type: object
properties:
  derives:
    type: array
    items:
      type: string
    default: ["Debug", "Clone", "PartialEq", "Serialize", "Deserialize"]
  serde_rename_strategy:
    type: string
    enum: [snake_case, camelCase, PascalCase, SCREAMING_SNAKE_CASE]
    default: snake_case
  visibility:
    type: string
    enum: [pub, pub(crate), ""]
    default: pub
  derive_hash:
    type: boolean
    description: Add Hash to derives
    default: false
  derive_copy:
    type: boolean
    description: Add Copy to derives
    default: false
```
