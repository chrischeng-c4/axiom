---
id: conductor-mamba-p0-remaining-spec
title: Conductor Mamba P0 Remaining
crate: mamba
files:
  - crates/mamba/src/runtime/class.rs
  - crates/mamba/src/parser/stmt_compound.rs
  - crates/mamba/src/parser/type_expr.rs
  - crates/mamba/tests/fixtures/cpython/test_grammar/classes.py
  - crates/mamba/tests/fixtures/cpython/test_grammar/generic_class_keywords.py
  - crates/mamba/tests/fixtures/cpython/test_grammar/type_alias_complex.py
  - crates/mamba/tests/fixtures/cpython/test_grammar/type_params_pep695.py
  - crates/mamba/tests/fixtures/cpython/test_match/match_basic.py
  - crates/mamba/tests/fixtures/cpython/test_match/match_sequence.py
  - crates/mamba/tests/fixtures/cpython/test_match/match_mapping.py
  - crates/mamba/tests/fixtures/cpython/test_match/match_class.py
status: source-of-truth
---

# Conductor Mamba P0 Remaining

This spec captures the remaining Mamba parser and runtime blockers before
Conductor can migrate to the Mamba engine. The prior P0 spec shipped import
aliases, relative imports, dict unpacking, and advanced f-strings. This follow-up
focuses on class-body annotations, subclass hooks, PEP 695 syntax combinations,
match/case XFAIL cleanup, and direct Conductor file checks.

## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: conductor-mamba-p0-remaining-requirements
title: Conductor Mamba P0 Remaining Requirements
requirements:
  R1:
    text: Bare type annotations parse and compile end-to-end in class bodies.
    type: functionalRequirement
    risk: Medium
    verification: Test
  R2:
    text: __init_subclass__ hook fires for direct base classes during class registration.
    type: functionalRequirement
    risk: High
    verification: Test
  R3:
    text: Class syntax supports PEP 695 type parameters combined with class keyword arguments.
    type: functionalRequirement
    risk: Medium
    verification: Test
  R4:
    text: PEP 695 type aliases and generic functions support ParamSpec, TypeVarTuple, and bounded parameters.
    type: functionalRequirement
    risk: Medium
    verification: Test
  R5:
    text: Stale match/case XFAIL annotations are removed only after parser tests pass.
    type: functionalRequirement
    risk: Low
    verification: Test
  R6:
    text: Conductor settings.py, models.py, and agent.py compile under cclab mamba check.
    type: functionalRequirement
    risk: High
    verification: Test
elements:
  parser_stmt_compound:
    text: parse_class and parse_match implementation
    type: module
    docref: crates/mamba/src/parser/stmt_compound.rs
  parser_type_expr:
    text: PEP 695 type parameter grammar
    type: module
    docref: crates/mamba/src/parser/type_expr.rs
  runtime_class:
    text: mb_class_register subclass hook dispatch
    type: module
    docref: crates/mamba/src/runtime/class.rs
  conductor_check:
    text: Conductor backend file compilation checks
    type: test
    docref: projects/conductor/be/src/
relationships:
  - { from: parser_stmt_compound, to: R1, type: satisfies }
  - { from: runtime_class, to: R2, type: satisfies }
  - { from: parser_stmt_compound, to: R3, type: satisfies }
  - { from: parser_type_expr, to: R4, type: satisfies }
  - { from: parser_stmt_compound, to: R5, type: satisfies }
  - { from: conductor_check, to: R6, type: verifies }
  - { from: R6, to: R1, type: derives }
  - { from: R6, to: R2, type: derives }
  - { from: R6, to: R3, type: derives }
  - { from: R6, to: R4, type: derives }
  - { from: R6, to: R5, type: derives }
---
requirementDiagram
    accDirection TB
    functionalRequirement R1 {
        id: "R1"
        text: "Bare type annotations parse and compile end-to-end in class bodies"
        risk: Medium
        verifymethod: Test
    }
    functionalRequirement R2 {
        id: "R2"
        text: "__init_subclass__ hook fires during class registration"
        risk: High
        verifymethod: Test
    }
    functionalRequirement R3 {
        id: "R3"
        text: "PEP 695 class type parameters combine with class keyword arguments"
        risk: Medium
        verifymethod: Test
    }
    functionalRequirement R4 {
        id: "R4"
        text: "PEP 695 ParamSpec TypeVarTuple and bounded parameters parse"
        risk: Medium
        verifymethod: Test
    }
    functionalRequirement R5 {
        id: "R5"
        text: "Match case XFAIL annotations removed after passing parser tests"
        risk: Low
        verifymethod: Test
    }
    functionalRequirement R6 {
        id: "R6"
        text: "Conductor backend target files compile under cclab mamba check"
        risk: High
        verifymethod: Test
    }
    element parser_stmt_compound {
        type: "module"
        docref: "crates/mamba/src/parser/stmt_compound.rs"
    }
    element parser_type_expr {
        type: "module"
        docref: "crates/mamba/src/parser/type_expr.rs"
    }
    element runtime_class {
        type: "module"
        docref: "crates/mamba/src/runtime/class.rs"
    }
    element conductor_check {
        type: "test"
        docref: "projects/conductor/be/src/"
    }
    parser_stmt_compound - satisfies -> R1
    runtime_class - satisfies -> R2
    parser_stmt_compound - satisfies -> R3
    parser_type_expr - satisfies -> R4
    parser_stmt_compound - satisfies -> R5
    conductor_check - verifies -> R6
    R6 - derives -> R1
    R6 - derives -> R2
    R6 - derives -> R3
    R6 - derives -> R4
    R6 - derives -> R5
```

## Dependency Model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: conductor-mamba-p0-dependencies
types:
  ClassBodyAnnotations: { kind: struct, label: "Stmt::BareAnnotation through parser, HIR, type check, and codegen" }
  InitSubclassHook: { kind: struct, label: "runtime::class mb_class_register hook dispatch" }
  ClassTypeParams: { kind: struct, label: "class Foo[T](metaclass=Meta) grammar" }
  TypeParamGrammar: { kind: struct, label: "ParamSpec, TypeVarTuple, bounded type parameters" }
  MatchParser: { kind: struct, label: "parse_match plus existing lowering" }
  ConductorTargets: { kind: struct, label: "settings.py, database/models.py, agents/llm/agent.py" }
edges:
  - { from: ConductorTargets, to: ClassBodyAnnotations, kind: references }
  - { from: ConductorTargets, to: InitSubclassHook, kind: references }
  - { from: ConductorTargets, to: ClassTypeParams, kind: references }
  - { from: ClassTypeParams, to: TypeParamGrammar, kind: references }
  - { from: ConductorTargets, to: MatchParser, kind: references, label: "suite gate" }
---
classDiagram
    class ClassBodyAnnotations
    class InitSubclassHook
    class ClassTypeParams
    class TypeParamGrammar
    class MatchParser
    class ConductorTargets
    ConductorTargets --> ClassBodyAnnotations : uses
    ConductorTargets --> InitSubclassHook : Pydantic / registries
    ConductorTargets --> ClassTypeParams : generics
    ClassTypeParams --> TypeParamGrammar : PEP695
    ConductorTargets --> MatchParser : suite gate
```

## Delivery Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: conductor-mamba-p0-delivery
entry: start
nodes:
  start: { kind: start, label: "Conductor Mamba P0 remaining work" }
  verify_bare_annotations: { kind: process, label: "Verify class-body bare annotations end-to-end" }
  implement_init_subclass: { kind: process, label: "Call base __init_subclass__ during class registration" }
  parse_class_params: { kind: process, label: "Support class type params with class keyword args" }
  parse_type_params: { kind: process, label: "Support ParamSpec, TypeVarTuple, bounded params" }
  clean_match_xfails: { kind: process, label: "Remove match/case XFAILs only after parse tests pass" }
  conductor_checks: { kind: process, label: "Run cclab mamba check on three Conductor files" }
  pass: { kind: terminal, label: "All P0 gates pass" }
edges:
  - { from: start, to: verify_bare_annotations }
  - { from: verify_bare_annotations, to: implement_init_subclass }
  - { from: implement_init_subclass, to: parse_class_params }
  - { from: parse_class_params, to: parse_type_params }
  - { from: parse_type_params, to: clean_match_xfails }
  - { from: clean_match_xfails, to: conductor_checks }
  - { from: conductor_checks, to: pass }
---
flowchart TD
    start([P0 remaining]) --> verify_bare_annotations[Verify class-body bare annotations]
    verify_bare_annotations --> implement_init_subclass[Dispatch __init_subclass__ on class registration]
    implement_init_subclass --> parse_class_params[Parse class type params plus metaclass keyword]
    parse_class_params --> parse_type_params[Parse ParamSpec TypeVarTuple bounded params]
    parse_type_params --> clean_match_xfails[Remove stale match/case XFAILs after tests pass]
    clean_match_xfails --> conductor_checks[Run Conductor mamba check targets]
    conductor_checks --> pass([All P0 gates pass])
```

## Conductor Check Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: conductor-mamba-check-flow
actors:
  - { id: Developer, kind: actor }
  - { id: MambaCli, kind: system, label: "cclab mamba check" }
  - { id: Parser, kind: system, label: "Mamba parser" }
  - { id: Runtime, kind: system, label: "runtime class hook" }
  - { id: Conductor, kind: system, label: "Conductor backend files" }
messages:
  - { from: Developer, to: MambaCli, name: "check settings.py / models.py / agent.py" }
  - { from: MambaCli, to: Conductor, name: "read Python source" }
  - { from: MambaCli, to: Parser, name: "parse class annotations, generics, type aliases" }
  - { from: MambaCli, to: Runtime, name: "validate class registration semantics for BaseModel-style hooks" }
  - { from: Runtime, to: MambaCli, name: "hook semantics available" }
  - { from: Parser, to: MambaCli, name: "AST accepted" }
  - { from: MambaCli, to: Developer, name: "exit 0" }
---
sequenceDiagram
    actor Developer
    participant MambaCli
    participant Conductor
    participant Parser
    participant Runtime
    Developer->>MambaCli: check target files
    MambaCli->>Conductor: read source
    MambaCli->>Parser: parse annotations and generics
    MambaCli->>Runtime: validate subclass hook semantics
    Runtime-->>MambaCli: hook available
    Parser-->>MambaCli: AST accepted
    MambaCli-->>Developer: exit 0
```

## Acceptance Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: bare-annotation-class
    given: class Point defines x and y as bare float annotations
    when: the class compiles through parse, HIR, type-check, and codegen
    then: no error is emitted and the Point section of cpython_compat classes passes
  - id: init-subclass-hook
    given: Base.__init_subclass__ appends the subclass name to a registry
    when: class Child(Base) is registered
    then: the hook receives cls=Child and the registry contains Child
  - id: class-type-params-with-metaclass
    given: class MyClass[T](metaclass=Meta) and Concrete[T: int](list[T], metaclass=Meta)
    when: parser tests run
    then: both class definitions produce valid AST and generic_class_keywords has no XFAIL
  - id: pep695-param-kinds
    given: type Callback[**P], type Shape[*Ts], and bounded generic function parameters
    when: PEP 695 parser tests run
    then: ParamSpecParam, TypeVarTupleParam, and bounded parameter nodes are represented
  - id: match-xfail-cleanup
    given: match_basic, match_sequence, match_mapping, and match_class fixtures
    when: their stale XFAIL comments are removed
    then: each parser test passes with no xfail log line
  - id: conductor-targets
    given: settings.py, database/models.py, and agents/llm/agent.py in Conductor
    when: cclab mamba check runs on each file
    then: all three commands exit 0
```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: conductor-mamba-p0-remaining-test-plan
title: Conductor Mamba P0 Remaining Test Plan
---
flowchart TD
    Runner["Verification commands"]
    Runner --> T1["cargo test -p mamba -- cpython_compat test_grammar/classes"]
    Runner --> T2["cargo test -p mamba -- jit::init_subclass_basic"]
    Runner --> T3["cargo test -p mamba -- cpython_compat test_grammar/generic_class_keywords"]
    Runner --> T4["cargo test -p mamba -- cpython_compat test_grammar/type_alias_complex"]
    Runner --> T5["cargo test -p mamba -- cpython_compat test_grammar/type_params_pep695"]
    Runner --> T6["cargo test -p mamba -- cpython_compat test_match/match_basic"]
    Runner --> T7["cargo test -p mamba -- cpython_compat test_match/match_sequence"]
    Runner --> T8["cargo test -p mamba -- cpython_compat test_match/match_mapping"]
    Runner --> T9["cargo test -p mamba -- cpython_compat test_match/match_class"]
    Runner --> T10["cclab mamba check projects/conductor/be/src/config/settings.py"]
    Runner --> T11["cclab mamba check projects/conductor/be/src/database/models.py"]
    Runner --> T12["cclab mamba check projects/conductor/be/src/agents/llm/agent.py"]
    Runner --> T13["cargo test -p mamba"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/class.rs
    action: modify
    impl_mode: hand-written
    description: Dispatch __init_subclass__ on direct base classes during class registration.
  - file: crates/mamba/src/parser/stmt_compound.rs
    action: modify
    impl_mode: hand-written
    description: Accept class type parameters combined with class keyword arguments and verify existing match parser coverage.
  - file: crates/mamba/src/parser/type_expr.rs
    action: modify
    impl_mode: hand-written
    description: Support PEP 695 ParamSpec, TypeVarTuple, and bounded type parameters.
  - file: crates/mamba/tests/fixtures/cpython/test_match/match_basic.py
    action: modify
    impl_mode: hand-written
    description: Remove stale XFAIL only after the parser test passes.
  - file: crates/mamba/tests/fixtures/cpython/test_match/match_sequence.py
    action: modify
    impl_mode: hand-written
    description: Remove stale XFAIL only after the parser test passes.
  - file: crates/mamba/tests/fixtures/cpython/test_match/match_mapping.py
    action: modify
    impl_mode: hand-written
    description: Remove stale XFAIL only after the parser test passes.
  - file: crates/mamba/tests/fixtures/cpython/test_match/match_class.py
    action: modify
    impl_mode: hand-written
    description: Remove stale XFAIL only after the parser test passes.
```
