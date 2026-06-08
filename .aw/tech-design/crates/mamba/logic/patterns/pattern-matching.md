---
id: mamba-pattern-matching
main_spec_ref: "cclab-mamba/pattern-matching.md"
title: Structural Pattern Matching
crate: mamba
---

# Structural Pattern Matching

## Overview
<!-- type: overview lang: markdown -->

This specification details the implementation of full structural pattern matching (PEP 634 match/case statements) within the Mamba compiler. The scope encompasses all five compiler layers to ensure complete support: Parser enhancements for all pattern types, formal Pattern representations (including AS patterns and mapping rest captures), Type checker narrowing within case branches, HIR and MIR lowering updates for decision tree compilation and nested patterns, and Cranelift IR code generation for pattern dispatch. The goal is to evolve the current skeletal structure to fully support literal, capture, wildcard, sequence, mapping, class, OR, AS, and guard constructs.

## Requirements
<!-- type: requirements lang: mermaid -->
```mermaid
---
id: pattern-matching-requirements
title: Structural Pattern Matching Requirements
---
requirementDiagram

requirement R1 {
  id: R1
  text: "Parser supports literal, capture, wildcard, sequence, mapping, class, OR, AS, and guard patterns"
  risk: high
  verifymethod: test
}
requirement R2 {
  id: R2
  text: "AS patterns bind the matched sub-pattern into a variable after the sub-pattern succeeds"
  risk: high
  verifymethod: test
}
requirement R3 {
  id: R3
  text: "Mapping patterns support rest capture syntax to bind unmatched key-value pairs"
  risk: high
  verifymethod: test
}
requirement R4 {
  id: R4
  text: "Type checking performs flow-sensitive type narrowing inside successful case branches"
  risk: high
  verifymethod: test
}
requirement R5 {
  id: R5
  text: "HIR contains explicit match and pattern nodes instead of wildcard placeholders"
  risk: high
  verifymethod: test
}
requirement R6 {
  id: R6
  text: "AST-to-HIR lowering preserves structural pattern semantics"
  risk: high
  verifymethod: test
}
requirement R7 {
  id: R7
  text: "HIR-to-MIR lowering emits recursive decision trees for nested patterns"
  risk: high
  verifymethod: test
}
requirement R8 {
  id: R8
  text: "Cranelift codegen emits dispatch for tests, captures, and guards"
  risk: high
  verifymethod: test
}
```

## Pattern Model
<!-- type: schema lang: yaml -->
```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "mamba://schemas/pattern-matching"
$defs:
  HirMatch:
    type: object
    properties:
      target: { description: "HIR expression being matched" }
      arms:
        type: array
        items: { $ref: "#/$defs/HirMatchArm" }
    required: [target, arms]
  HirMatchArm:
    type: object
    properties:
      pattern: { $ref: "#/$defs/HirPattern" }
      guard: { description: "Optional guard expression" }
      body: { description: "Case body statements" }
    required: [pattern, body]
  HirPattern:
    type: object
    properties:
      kind:
        type: string
        enum: [literal, capture, wildcard, sequence, mapping, class, or, as]
      rest_capture:
        type: string
        description: "Mapping or sequence rest capture name when present"
      subpatterns:
        type: array
        items: { $ref: "#/$defs/HirPattern" }
    required: [kind]
```

## Decision-Tree Lowering
<!-- type: logic lang: mermaid -->
```mermaid
---
id: pattern-decision-tree
entry: parse_match
nodes:
  parse_match: { kind: start, label: "Parse match/case statement" }
  build_ast: { kind: process, label: "Build AST pattern nodes" }
  lower_hir: { kind: process, label: "Lower AST patterns to HirPattern variants" }
  narrow_types: { kind: process, label: "Apply case-local type narrowing" }
  build_tree: { kind: process, label: "Lower HIR patterns to MIR decision tree" }
  is_mapping: { kind: decision, label: "Pattern includes mapping rest capture?" }
  bind_rest: { kind: process, label: "Bind unmatched mapping entries to rest variable" }
  emit_codegen: { kind: process, label: "Emit Cranelift checks, captures, and guard branches" }
  done: { kind: terminal, label: "Pattern dispatch executable" }
edges:
  - { from: parse_match, to: build_ast }
  - { from: build_ast, to: lower_hir }
  - { from: lower_hir, to: narrow_types }
  - { from: narrow_types, to: build_tree }
  - { from: build_tree, to: is_mapping }
  - { from: is_mapping, to: bind_rest, label: "yes" }
  - { from: is_mapping, to: emit_codegen, label: "no" }
  - { from: bind_rest, to: emit_codegen }
  - { from: emit_codegen, to: done }
---
flowchart TD
    parse_match([parse match/case]) --> build_ast[AST pattern nodes]
    build_ast --> lower_hir[HirPattern variants]
    lower_hir --> narrow_types[type narrowing]
    narrow_types --> build_tree[MIR decision tree]
    build_tree --> is_mapping{mapping rest?}
    is_mapping -->|yes| bind_rest[bind rest]
    is_mapping -->|no| emit_codegen[Cranelift dispatch]
    bind_rest --> emit_codegen
    emit_codegen --> done([executable dispatch])
```

## Scenarios
<!-- type: scenarios lang: yaml -->
```yaml
scenarios:
  - id: literal-capture
    when: a match statement has literal cases followed by a fallback capture
    then: dispatch compares literal equality and binds the fallback target variable
  - id: class-type-narrowing
    when: a case branch uses a class pattern with positional and keyword fields
    then: the matched value is narrowed to the class type within that branch
  - id: mapping-rest-capture
    when: a mapping pattern includes rest capture syntax
    then: HirPattern::Mapping records fixed keys and binds remaining entries to the rest variable
  - id: nested-or-patterns
    when: a nested sequence pattern contains OR alternatives
    then: MIR lowering builds a recursive decision tree with alternative paths
```

## Test Plan
<!-- type: test-plan lang: mermaid -->
```mermaid
---
id: pattern-matching-test-plan
title: Structural Pattern Matching Test Plan
---
flowchart TD
    Parser["parser tests for literal, capture, wildcard, sequence, mapping, class, or, as, guard"]
    Parser --> Hir["HIR lowering tests preserve HirPattern variants"]
    Hir --> Types["type checker tests branch-local narrowing"]
    Types --> Mir["MIR lowering tests nested decision trees"]
    Mir --> Codegen["runtime/codegen tests execute pattern dispatch"]
```

## Changes
<!-- type: changes lang: yaml -->
```yaml
changes:
  - file: crates/mamba/src/parser/
    action: modify
    impl_mode: hand-written
    description: Parse AS patterns and mapping rest captures.
  - file: crates/mamba/src/hir/
    action: modify
    impl_mode: hand-written
    description: Add HirMatch statements and HirPattern variants for PEP 634 forms.
  - file: crates/mamba/src/lower/ast_to_hir.rs
    action: modify
    impl_mode: hand-written
    description: Lower match/case AST constructs into HirMatch and HirPattern.
  - file: crates/mamba/src/types/
    action: modify
    impl_mode: hand-written
    description: Apply case-branch type narrowing for successful pattern matches.
  - file: crates/mamba/src/lower/hir_to_mir.rs
    action: modify
    impl_mode: hand-written
    description: Generate recursive MIR decision trees for nested patterns and guards.
  - file: crates/mamba/src/codegen/
    action: modify
    impl_mode: hand-written
    description: Emit Cranelift dispatch for tests, captures, and guard evaluation.
```
