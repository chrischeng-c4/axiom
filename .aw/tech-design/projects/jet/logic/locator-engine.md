---
id: projects-jet-logic-locator-engine-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Jet Locator Engine

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/locator-engine.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# Jet Locator Engine

### Overview

This spec owns the current Rust-side Jet locator engine in
`crates/jet/src/browser/locator.rs`. A locator is a lazy, cloneable DOM query
plan built from selector steps, text filters, positional indexing, and per
locator timeout options. Construction performs no CDP I/O; reads and actions
compile the plan into JavaScript and execute it through `Runtime.evaluate` on a
`CdpSession`.

The contract describes the implementation as it exists today. Role lookup is a
best-effort DOM query over explicit `role` attributes plus a small implicit tag
map, not a full accessibility tree query. Actions use in-page JavaScript such as
`el.click()` and synthetic events, not coordinate-level input dispatch.

| Area | Source | Responsibility |
|------|--------|----------------|
| Locator type | `crates/jet/src/browser/locator.rs` | Holds session, selector steps, filters, index, options |
| Selector parser | `crates/jet/src/browser/locator.rs` | Parses CSS, `role=...`, `role=...[name="..."]`, and `text=...` |
| Query compiler | `crates/jet/src/browser/locator.rs` | Builds collection, count, and single element JS expressions |
| Auto-wait | `crates/jet/src/browser/locator.rs` | Polls actionability stages until target state or timeout |
| Actions | `crates/jet/src/browser/locator.rs` | Click, fill, check, uncheck, and hover through page JS evaluation |
| Reads | `crates/jet/src/browser/locator.rs` | Count, text, inner text, attribute, visibility, and wait_for |

### Requirements

```mermaid
---
id: jet-locator-engine-requirements
entry: L1
---
requirementDiagram
    requirement L1 {
        id: L1
        text: Locator construction and chaining are lazy and send no CDP request
        risk: medium
        verifymethod: test
    }
    requirement L2 {
        id: L2
        text: SelectorExpr parse supports CSS role role name and text selector forms
        risk: high
        verifymethod: test
    }
    requirement L3 {
        id: L3
        text: Chained locators append selector steps and reset filters and index
        risk: medium
        verifymethod: test
    }
    requirement L4 {
        id: L4
        text: Text filters and positional indexes are applied after selector collection
        risk: high
        verifymethod: test
    }
    requirement L5 {
        id: L5
        text: Count evaluates the current collection length without auto wait
        risk: medium
        verifymethod: test
    }
    requirement L6 {
        id: L6
        text: Text and attribute reads wait only for Attached before evaluating the read
        risk: high
        verifymethod: test
    }
    requirement L7 {
        id: L7
        text: Actions wait for Actionable before evaluating page JavaScript
        risk: high
        verifymethod: test
    }
    requirement L8 {
        id: L8
        text: Actionability probes Detached Attached Stable and Actionable from current DOM state
        risk: high
        verifymethod: test
    }
    requirement L9 {
        id: L9
        text: Timeout errors include last reached actionability state selector description and timeout
        risk: medium
        verifymethod: test
    }
    requirement L10 {
        id: L10
        text: Runtime evaluate errors are returned as typed locator errors
        risk: medium
        verifymethod: test
    }
```

### Scenarios

```yaml
scenarios:
  - id: S1
    requirement: L1
    title: Page creates a root locator and additional locator calls only clone query state
  - id: S2
    requirement: L2
    title: Parser accepts CSS role text and role name selectors and rejects empty strings
  - id: S3
    requirement: L3
    title: Child locator appends a selector step and clears previous filter and index state
  - id: S4
    requirement: L4
    title: filter_has_text narrows the last step collection before nth first or last is selected
  - id: S5
    requirement: L5
    title: count returns the collection length from one Runtime.evaluate call without wait polling
  - id: S6
    requirement: L6
    title: text_content inner_text and get_attribute wait for Attached and then evaluate a suffix read
  - id: S7
    requirement: L7
    title: click fill check uncheck and hover wait for Actionable then run JavaScript in the page
  - id: S8
    requirement: L8
    title: Hidden or detached elements remain below the requested actionability rank until timeout
  - id: S9
    requirement: L9
    title: Timeout reports the selector chain including filters and index labels
  - id: S10
    requirement: L10
    title: CDP send failures and JavaScript exception details map into LocatorError variants
```

### Interaction

```mermaid
---
id: jet-locator-engine-interaction
entry: Caller
---
sequenceDiagram
    participant Caller
    participant Locator
    participant Compiler as JS compiler
    participant Session as CdpSession
    participant Page as Page Runtime

    Caller->>Locator: locator("button").filter_has_text("Save").first()
    Locator-->>Caller: cloned Locator
    Caller->>Locator: click()
    Locator->>Locator: wait_until(Actionable)
    loop until rank is high enough or timeout
        Locator->>Compiler: compile_collection_expr()
        Compiler-->>Locator: JavaScript collection expression
        Locator->>Session: Runtime.evaluate probe_state expression
        Session->>Page: evaluate
        Page-->>Session: actionability string
        Session-->>Locator: state value
    end
    Locator->>Compiler: compile_single_expr()
    Compiler-->>Locator: JavaScript single element expression
    Locator->>Session: Runtime.evaluate action expression
    Session->>Page: evaluate el.click or event code
    Page-->>Session: result or exception
    Session-->>Locator: typed result
    Locator-->>Caller: Result
```

### Logic

```mermaid
---
id: jet-locator-engine-logic
entry: A
---
flowchart TD
    A[Selector input] --> B{prefix}
    B -->|role=| C[parse role and optional name predicate]
    B -->|text=| D[create Text selector]
    B -->|empty| E[InvalidSelector]
    B -->|other| F[create Css selector]

    C --> G[append selector step]
    D --> G
    F --> G
    G --> H{operation}
    H -->|chain| I[clone locator append step clear filter and index]
    H -->|filter_has_text| J[clone locator append HasText filter]
    H -->|first last nth| K[clone locator set Index]
    H -->|count| L[compile collection length and evaluate once]
    H -->|read| M[wait until Attached]
    H -->|action| N[wait until Actionable]

    M --> O[compile single expression and suffix read]
    N --> P[compile single expression and action JS]
    O --> Q[Runtime.evaluate]
    P --> Q
    L --> Q

    Q --> R{CDP result}
    R -->|exceptionDetails| S[LocatorError EvalError]
    R -->|send error| T[LocatorError CdpError]
    R -->|value| U[return typed value]

    N --> V[probe_state]
    M --> V
    V --> W{collection state}
    W -->|empty or disconnected| X[Detached]
    W -->|not visible| Y[Attached]
    W -->|disabled| Z[Stable]
    W -->|visible and enabled| AA[Actionable]
    X --> AB{timeout exceeded}
    Y --> AB
    Z --> AB
    AA --> AC[target reached]
    AB -->|yes| AD[LocatorError Timeout]
    AB -->|no| V
```

### Dependency Model

```mermaid
---
id: jet-locator-engine-dependencies
entry: Locator
---
classDiagram
    class Locator {
        -CdpSession session
        -Vec~SelectorExpr~ steps
        -Vec~Filter~ filters
        -Option~Index~ index
        -LocatorOptions opts
        +locator(selector) LocatorResult~Locator~
        +filter_has_text(text) Locator
        +nth(i) Locator
        +first() Locator
        +last() Locator
        +get_by_role(role name) Locator
        +get_by_text(text) Locator
        +with_options(opts) Locator
        +count() LocatorResult~usize~
        +text_content() LocatorResult~String~
        +inner_text() LocatorResult~String~
        +get_attribute(name) LocatorResult~Option~String~~
        +is_visible() LocatorResult~bool~
        +wait_for(state) LocatorResult~unit~
        +click() LocatorResult~unit~
        +fill(text) LocatorResult~unit~
        +check() LocatorResult~unit~
        +uncheck() LocatorResult~unit~
        +hover() LocatorResult~unit~
    }
    class SelectorExpr {
        <<enum>>
        Css
        Role
        Text
    }
    class Filter {
        <<enum>>
        HasText
    }
    class Index {
        <<enum>>
        Nth
        First
        Last
    }
    class Actionability {
        <<enum>>
        Detached
        Attached
        Visible
        Stable
        Actionable
    }
    class LocatorOptions {
        +timeout_ms
        +poll_interval_ms
        +text_substring
        +text_case_insensitive
    }
    class LocatorError {
        <<enum>>
        Timeout
        Ambiguous
        InvalidSelector
        CdpError
        EvalError
    }
    class CdpSession {
        +send(method params)
    }

    Locator --> CdpSession
    Locator --> SelectorExpr
    Locator --> Filter
    Locator --> Index
    Locator --> LocatorOptions
    Locator --> Actionability
    Locator --> LocatorError
```

### Locator API

```yaml
openrpc: 1.3.2
info:
  title: Jet Rust Locator API
  version: 0.1.0
  description: Lazy DOM locator API executed through Runtime.evaluate.
methods:
  - name: locator
    summary: Append a child selector step and return a cloned locator.
    params:
      - name: selector
        schema:
          type: string
    result:
      name: locator
      schema:
        type: object
        x-rust-type: Locator
    x-sdd:
      wait: none
      errors:
        - InvalidSelector
  - name: filter_has_text
    summary: Append a substring text filter to the current collection.
    params:
      - name: text
        schema:
          type: string
    result:
      name: locator
      schema:
        type: object
        x-rust-type: Locator
    x-sdd:
      wait: none
  - name: first
    summary: Select the first element in the filtered collection.
    params: []
    result:
      name: locator
      schema:
        type: object
        x-rust-type: Locator
    x-sdd:
      wait: none
  - name: nth
    summary: Select the indexed element in the filtered collection.
    params:
      - name: i
        schema:
          type: integer
    result:
      name: locator
      schema:
        type: object
        x-rust-type: Locator
    x-sdd:
      wait: none
  - name: count
    summary: Evaluate the current collection length without auto-wait.
    params: []
    result:
      name: count
      schema:
        type: integer
        minimum: 0
    x-sdd:
      wait: none
      cdp_methods:
        - Runtime.evaluate
  - name: text_content
    summary: Wait for Attached and return textContent or an empty string.
    params: []
    result:
      name: text
      schema:
        type: string
    x-sdd:
      wait: attached
      cdp_methods:
        - Runtime.evaluate
  - name: get_attribute
    summary: Wait for Attached and return an attribute value.
    params:
      - name: name
        schema:
          type: string
    result:
      name: value
      schema:
        type: string
        nullable: true
    x-sdd:
      wait: attached
      cdp_methods:
        - Runtime.evaluate
  - name: click
    summary: Wait for Actionable and evaluate el.click in the page.
    params: []
    result:
      name: unit
      schema:
        type: "null"
    x-sdd:
      wait: actionable
      cdp_methods:
        - Runtime.evaluate
  - name: fill
    summary: Wait for Actionable then set input or textarea value and dispatch input and change events.
    params:
      - name: text
        schema:
          type: string
    result:
      name: unit
      schema:
        type: "null"
    x-sdd:
      wait: actionable
      cdp_methods:
        - Runtime.evaluate
```

### Schema

```yaml
schemas:
  SelectorExpr:
    rust_type: SelectorExpr
    variants:
      - name: Css
        fields:
          selector:
            type: string
            parse_rule: Any non-empty selector without a recognized prefix.
      - name: Role
        fields:
          role:
            type: string
          name:
            type: string
            nullable: true
        parse_rule: role=<role> or role=<role>[name="<name>"].
      - name: Text
        fields:
          text:
            type: string
        parse_rule: text=<query>.
    validation:
      - Empty selector strings return LocatorError::InvalidSelector.
      - Role predicates other than name return LocatorError::InvalidSelector.
      - Unclosed role predicate brackets return LocatorError::InvalidSelector.
  LocatorOptions:
    rust_type: LocatorOptions
    fields:
      timeout_ms:
        type: u64
        default: 5000
      poll_interval_ms:
        type: u64
        default: 100
      text_substring:
        type: bool
        default: true
      text_case_insensitive:
        type: bool
        default: true
  Filter:
    rust_type: Filter
    variants:
      - name: HasText
        fields:
          text:
            type: string
  Index:
    rust_type: Index
    variants:
      - name: Nth
        fields:
          i:
            type: i32
            note: Negative indexes count from the end.
      - name: First
      - name: Last
  Actionability:
    rust_type: Actionability
    ordered_values:
      - Detached
      - Attached
      - Visible
      - Stable
      - Actionable
  LocatorError:
    rust_type: LocatorError
    variants:
      - Timeout
      - Ambiguous
      - InvalidSelector
      - CdpError
      - EvalError
```

### Test Plan

```mermaid
---
id: jet-locator-engine-test-plan
entry: T1
---
flowchart TD
    T1[cargo test -p jet browser::locator::tests] --> T2[SelectorExpr parse coverage]
    T2 --> T3[Invalid selector coverage]
    T3 --> T4[json_str escaping coverage]
    T4 --> T5[actionability_rank monotonic coverage]
    T5 --> T6[implicit role tag map coverage]
    T6 --> T7[compile_step CSS role and text JS coverage]
    T7 --> T8[aw td check logic locator-engine spec]
```

### Changes

```yaml
changes:
  - path: .aw/tech-design/crates/jet/logic/locator-engine.md
    action: create
    purpose: Re-home the current Rust locator engine TD under the logic spec directory.
    impl_mode: hand-written
  - path: .aw/tech-design/crates/jet/testing/locator-engine.md
    action: delete
    purpose: Remove the stale testing-directory TD that described future behavior as current behavior.
    impl_mode: hand-written
  - path: crates/jet/src/browser/locator.rs
    action: none
    purpose: Existing implementation remains the source described by this spec.
    impl_mode: hand-written
```
