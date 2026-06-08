---
id: exception
title: Exception Runtime
crate: mamba
files:
  - crates/mamba/src/runtime/exception.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: dbbaf7396
---

# Exception Runtime

Mamba's exception subsystem. One `MbException` value type plus a
thread-local pending-exception slot drives every Python control-flow
path that involves `raise` / `try` / `except` / `except*`. Two cross-cuts
are this spec's load-bearing invariants:

1. **`mb_raise("StopIteration")` mirrors the iterator flag**
   (`iter::signal_stop_iteration`) — `iter.md` consumes both.
2. **`raise X from Y` always sets `suppress_context = True`**, regardless
   of whether `Y` is None — controls how the printer walks the chain.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: exception-types
types:
  MbException:       { kind: struct }
  ExceptionHandler:  { kind: struct }
  CurrentException:  { kind: struct, label: "thread_local Option<MbException>" }
  HandlerStack:      { kind: struct, label: "thread_local Vec<ExceptionHandler>" }
  ClassRegistry:     { kind: struct, label: "from runtime::class" }
  IterFlag:          { kind: struct, label: "STOP_ITERATION from runtime::iter" }
  MbValue:           { kind: struct }
edges:
  - { from: CurrentException, to: MbException, kind: owns,       label: "0..1 pending" }
  - { from: HandlerStack,     to: ExceptionHandler, kind: owns,  label: "try-frame stack" }
  - { from: MbException,      to: MbException, kind: references, label: "cause / context" }
  - { from: MbException,      to: MbValue,     kind: references, label: "stored as Instance via store_exception_as_value" }
  - { from: MbException,      to: ClassRegistry, kind: references, label: "MRO via is_subclass_of + check_class_hierarchy" }
  - { from: MbException,      to: IterFlag,    kind: references, label: "StopIteration mirrors iter::signal_stop_iteration" }
---
classDiagram
    class MbException
    class ExceptionHandler
    class CurrentException
    class HandlerStack
    class ClassRegistry
    class IterFlag
    class MbValue
    CurrentException --> MbException : owns
    HandlerStack --> ExceptionHandler : owns
    MbException --> MbException : cause / context
    MbException --> MbValue : stored as Instance
    MbException --> ClassRegistry : MRO
    MbException --> IterFlag : StopIteration mirror
```

## Exception state shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "exception-types"
$defs:
  MbException:
    type: object
    x-rust-type: MbException
    properties:
      exc_type:         { type: string, description: "exception class name (e.g. ValueError)" }
      message:          { type: string }
      cause:
        oneOf:
          - { type: "null" }
          - { $ref: "#/$defs/MbException" }
        description: "raise X from Y — explicit chain"
      context:
        oneOf:
          - { type: "null" }
          - { $ref: "#/$defs/MbException" }
        description: "active exception when this was raised — implicit chain"
      suppress_context:
        type: boolean
        description: "set by raise-from (always true) or raise-from-None — printer skips context"
      traceback:
        type: array
        items:
          type: object
          properties:
            file:     { type: string }
            line:     { type: integer, x-rust-type: u32 }
            function: { type: string }
          required: [file, line, function]
    required: [exc_type, message, cause, context, suppress_context, traceback]
  ExceptionHandler:
    type: object
    x-rust-type: ExceptionHandler
    description: "try-frame entry"
    properties:
      catch_types:
        type: array
        items: { type: string }
        description: "empty = catch all"
      has_finally: { type: boolean }
    required: [catch_types, has_finally]
```

## Exception lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: exception-lifecycle
initial: Idle
nodes:
  Idle:     { kind: initial,  label: "no pending exception" }
  Pending:  { kind: normal,   label: "CURRENT_EXCEPTION = Some(exc)" }
  Caught:   { kind: normal,   label: "value extracted; slot empty" }
  Uncaught: { kind: terminal, label: "module exit reports Traceback" }
edges:
  - { from: Idle,    to: Pending,  event: "mb_raise / mb_raise_from / mb_raise_with_context / mb_raise_from_with_context / mb_reraise" }
  - { from: Pending, to: Caught,   event: mb_catch_exception, guard: "matching except handler" }
  - { from: Pending, to: Idle,     event: "mb_clear_exception | iter clears for StopIteration leak" }
  - { from: Pending, to: Uncaught, event: mb_take_uncaught_traceback }
  - { from: Caught,  to: Pending,  event: "mb_reraise (re-raise from except)" }
  - { from: Caught,  to: Idle,     event: "handler completes" }
---
stateDiagram-v2
    [*] --> Idle
    Idle --> Pending: mb_raise*
    Pending --> Caught: mb_catch_exception [match]
    Pending --> Idle: mb_clear_exception
    Pending --> Uncaught: mb_take_uncaught_traceback
    Caught --> Pending: mb_reraise
    Caught --> Idle: handler done
    Uncaught --> [*]
```

## Raise dispatch
<!-- type: logic lang: mermaid -->

```mermaid
---
id: raise-dispatch
entry: enter
nodes:
  enter:           { kind: start,    label: "raise / raise from / raise from-context / reraise" }
  is_stop:         { kind: decision, label: "exc_type == StopIteration?" }
  signal_iter:     { kind: process,  label: "iter::signal_stop_iteration" }
  classify:        { kind: decision, label: "raise variant?" }
  build_plain:     { kind: process,  label: "MbException::new(exc_type, msg)" }
  build_from:      { kind: process,  label: "build + cause + suppress_context = true" }
  build_ctx:       { kind: process,  label: "build + context (active exception)" }
  build_from_ctx:  { kind: process,  label: "build + cause + context + suppress_context = true" }
  reraise:         { kind: process,  label: "delegate to class::mb_raise_instance (preserves Instance fields)" }
  store:           { kind: process,  label: "CURRENT_EXCEPTION = Some(exc)" }
  done:            { kind: terminal, label: "return ()" }
edges:
  - { from: enter,           to: is_stop }
  - { from: is_stop,         to: signal_iter, label: "yes" }
  - { from: signal_iter,     to: classify }
  - { from: is_stop,         to: classify,    label: "no" }
  - { from: classify,        to: build_plain,    label: "raise" }
  - { from: classify,        to: build_from,     label: "raise from" }
  - { from: classify,        to: build_ctx,      label: "raise (in except)" }
  - { from: classify,        to: build_from_ctx, label: "raise from + context" }
  - { from: classify,        to: reraise,        label: "reraise" }
  - { from: build_plain,     to: store }
  - { from: build_from,      to: store }
  - { from: build_ctx,       to: store }
  - { from: build_from_ctx,  to: store }
  - { from: reraise,         to: done }
  - { from: store,           to: done }
---
flowchart TD
    enter([raise / from / context / reraise]) --> is_stop{StopIteration?}
    is_stop -->|yes| signal_iter[iter signal_stop_iteration]
    signal_iter --> classify
    is_stop -->|no| classify{variant}
    classify -->|raise| build_plain[plain MbException]
    classify -->|raise from| build_from[suppress_context = true; cause]
    classify -->|raise in except| build_ctx[attach context]
    classify -->|raise from + ctx| build_from_ctx[suppress_context = true; cause + context]
    classify -->|reraise| reraise[class mb_raise_instance]
    build_plain --> store[set CURRENT_EXCEPTION]
    build_from --> store
    build_ctx --> store
    build_from_ctx --> store
    reraise --> done([return])
    store --> done
```

## Try-except interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: try-except-emit
actors:
  - { id: JIT,       kind: system, label: "JIT-compiled try block" }
  - { id: Exception, kind: system, label: "exception.rs" }
  - { id: ClassReg,  kind: system, label: "class.rs registry" }
messages:
  - { from: JIT,       to: Exception, name: mb_push_handler(catch_all) }
  - { from: JIT,       to: JIT,       name: "execute try body" }
  - { from: JIT,       to: Exception, name: mb_raise(ValueError, msg) }
  - { from: Exception, to: Exception, name: "CURRENT_EXCEPTION = Some" }
  - { from: JIT,       to: Exception, name: mb_has_exception, returns: bool }
  - { from: JIT,       to: Exception, name: mb_exception_matches(exc, ValueError) }
  - { from: Exception, to: ClassReg,  name: is_subclass_of }
  - { from: ClassReg,  to: Exception, name: bool, returns: bool }
  - { from: Exception, to: JIT,       name: matched, returns: bool }
  - { from: JIT,       to: Exception, name: mb_catch_exception, returns: MbValue }
  - { from: JIT,       to: JIT,       name: "execute except body" }
  - { from: JIT,       to: Exception, name: mb_clear_exception }
  - { from: JIT,       to: Exception, name: mb_pop_handler }
---
sequenceDiagram
    participant JIT
    participant Exception
    participant ClassReg
    JIT->>Exception: mb_push_handler
    Note over JIT: execute try body
    JIT->>Exception: mb_raise(ValueError, msg)
    Exception->>Exception: CURRENT_EXCEPTION = Some
    JIT->>Exception: mb_has_exception
    Exception-->>JIT: true
    JIT->>Exception: mb_exception_matches(exc, ValueError)
    Exception->>ClassReg: is_subclass_of
    ClassReg-->>Exception: true
    Exception-->>JIT: matched
    JIT->>Exception: mb_catch_exception
    Exception-->>JIT: MbValue (Instance)
    Note over JIT: execute except body
    JIT->>Exception: mb_clear_exception
    JIT->>Exception: mb_pop_handler
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: keyerror-repr
    given: exceptions/keyerror_repr.py catches a missing dict key
    when: the exception is printed with str and repr
    then: formatting adds exactly one layer of CPython-compatible quoting
  - id: raise-from-context
    given: exceptions/chaining.py raises one exception from another
    when: raise X from Y executes
    then: cause is set and suppress_context is true so implicit context is not printed
  - id: except-star-split
    given: exceptions/except_star_basic.py raises an ExceptionGroup
    when: except* handles one subtype
    then: matched exceptions run the handler and unmatched rest re-propagates
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-exception-test-plan
title: Exception Runtime Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> KeyError["exceptions/keyerror_repr.py"]
    Runner --> Args["exceptions/args_and_str.py"]
    Runner --> Chaining["exceptions/chaining.py"]
    Runner --> Finally["exceptions/try_finally.py"]
    Runner --> ExceptTuple["exceptions/except_tuple.py"]
    Runner --> ExceptStar["exceptions/except_star_basic.py"]
    Runner --> StopIter["iterators/custom_iter_non_self.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/exception.rs
    action: modify
    impl_mode: hand-written
    description: "Exception object, thread-local pending slot, raise variants, ExceptionGroup, except*, MRO matching. Hand-written; spec is the design contract."
```
