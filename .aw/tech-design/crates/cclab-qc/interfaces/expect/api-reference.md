---
id: expect-api-reference
main_spec_ref: "cclab-qc/interfaces/expect/api-reference.md"
fill_sections: [overview, requirements, scenarios, schema, doc, changes]
---

# Expect Assertion API Reference

## Overview
<!-- type: overview lang: markdown -->

`cclab-qc` exposes an expect-style assertion API from
`crates/cclab-qc/src/assertions.rs`. The public entry point is
`expect(value) -> Expectation<T>`. Assertions return `AssertionResult`, which
is `Result<(), AssertionError>`, so runner integrations can preserve structured
failure messages, expected values, actual values, and assertion type.

The current implementation supports equality, negation, boolean checks, option
checks, numeric ordering, string matching, vector membership/length, and simple
JSON object/path checks. JSON paths use dot notation such as `user.name`.

## Requirements
<!-- type: schema lang: yaml -->

```yaml
requirements:
  - id: R1
    title: Fluent expectation builder
    priority: must
    statement: "The API must wrap any value in `Expectation<T>` through the `expect` entry point."
    implementation:
      - "`Expectation::new(value)` stores the value and starts in non-negated mode."
      - "`Expectation::not()` toggles negation for the next assertion."
      - "`value()` and `is_negated()` expose read-only inspection."

  - id: R2
    title: Structured assertion errors
    priority: must
    statement: "Failed assertions must return structured errors suitable for runner reports."
    implementation:
      - "Use `AssertionError { message, expected, actual, assertion_type }`."
      - "Populate expected and actual values when the assertion has meaningful comparison data."

  - id: R3
    title: Core matcher families
    priority: must
    statement: "The API must provide matchers for the common scalar, option, string, vector, and JSON cases used by qc tests."
    implementation:
      - "Equality: `to_equal`, `to_not_equal`."
      - "Boolean: `to_be_true`, `to_be_false`."
      - "Option: `to_be_some`, `to_be_none`."
      - "Numeric/order: `to_be_greater_than`, `to_be_less_than`, `to_be_at_least`, `to_be_at_most`, `to_be_between`."
      - "String: `to_contain`, `to_start_with`, `to_end_with`, `to_match`, `to_have_length`, `to_be_empty`."
      - "Vector: `to_contain_item`, `to_have_length`, `to_be_empty`."
      - "JSON: `to_have_key`, `to_have_keys`, `to_have_path_value`."

  - id: R4
    title: Regex validation
    priority: should
    statement: "String regex assertions must fail with an assertion error when the regex pattern is invalid."
    implementation:
      - "`to_match` compiles the pattern with `regex::Regex::new`."
      - "Regex compile errors are returned as `AssertionError` with assertion type `to_match`."
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - name: Equality assertion passes
    given:
      - "An expectation created with `expect(42)`."
    when:
      - "`to_equal(&42)` is called."
    then:
      - "The assertion returns `Ok(())`."

  - name: Negated equality passes
    given:
      - "An expectation created with `expect(42).not()`."
    when:
      - "`to_equal(&43)` is called."
    then:
      - "The assertion returns `Ok(())`."

  - name: Option assertion failure
    given:
      - "An expectation created with `expect(None::<i32>)`."
    when:
      - "`to_be_some()` is called."
    then:
      - "The assertion returns `AssertionError` with assertion type `to_be_some`."

  - name: JSON dot path assertion
    given:
      - "A JSON object containing `{ \"user\": { \"name\": \"Alice\" } }`."
    when:
      - "`to_have_path_value(\"user.name\", &json!(\"Alice\"))` is called."
    then:
      - "The assertion returns `Ok(())`."
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
api:
  entrypoint:
    name: expect
    signature: "expect<T>(value: T) -> Expectation<T>"
    module: crates/cclab-qc/src/assertions.rs

  result:
    name: AssertionResult
    alias: "Result<(), AssertionError>"

  error:
    name: AssertionError
    fields:
      message: String
      expected: "Option<String>"
      actual: "Option<String>"
      assertion_type: String

  expectation:
    name: Expectation<T>
    fields:
      value: T
      negated: bool
    methods:
      common:
        - "new(value: T) -> Self"
        - "not(self) -> Self"
        - "value(&self) -> &T"
        - "is_negated(&self) -> bool"
      equality:
        - "to_equal(&self, expected: &T) -> AssertionResult"
        - "to_not_equal(&self, expected: &T) -> AssertionResult"
      boolean:
        - "to_be_true(&self) -> AssertionResult"
        - "to_be_false(&self) -> AssertionResult"
      option:
        - "to_be_some(&self) -> AssertionResult"
        - "to_be_none(&self) -> AssertionResult"
      numeric:
        - "to_be_greater_than(&self, expected: &T) -> AssertionResult"
        - "to_be_less_than(&self, expected: &T) -> AssertionResult"
        - "to_be_at_least(&self, expected: &T) -> AssertionResult"
        - "to_be_at_most(&self, expected: &T) -> AssertionResult"
        - "to_be_between(&self, low: T, high: T) -> AssertionResult"
      string:
        - "to_contain(&self, substring: &str) -> AssertionResult"
        - "to_start_with(&self, prefix: &str) -> AssertionResult"
        - "to_end_with(&self, suffix: &str) -> AssertionResult"
        - "to_match(&self, pattern: &str) -> AssertionResult"
        - "to_have_length(&self, length: usize) -> AssertionResult"
        - "to_be_empty(&self) -> AssertionResult"
      vector:
        - "to_contain_item(&self, item: &T) -> AssertionResult"
        - "to_have_length(&self, length: usize) -> AssertionResult"
        - "to_be_empty(&self) -> AssertionResult"
      json:
        - "to_have_key(&self, key: &str) -> AssertionResult"
        - "to_have_keys(&self, keys: &[&str]) -> AssertionResult"
        - "to_have_path_value(&self, path: &str, expected: &JsonValue) -> AssertionResult"
```

## Current Gaps
<!-- type: doc lang: markdown -->

The current `assertions.rs` implementation does not expose Result-specific
matchers, approximate floating-point equality, JSONPath syntax with a leading
`$`, or a custom matcher registry. Those should be covered by separate issue
and TD work before becoming contract requirements.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .aw/tech-design/crates/cclab-qc/interfaces/expect/api-reference.md
    action: move
    section: overview
    impl_mode: hand-written
    description: "Move the expect assertion API reference out of the crate spec root and align it with the current assertions.rs API."
  - path: .aw/tech-design/crates/cclab-qc/README.md
    action: modify
    section: doc
    impl_mode: hand-written
    description: "Link the normalized expect assertion API reference from the crate spec index."
  - path: crates/cclab-qc/src/assertions.rs
    action: reference
    section: schema
    impl_mode: hand-written
    description: "Defines expect, Expectation, AssertionResult, AssertionError, and matcher methods."
  - path: crates/cclab-qc/src/lib.rs
    action: reference
    section: schema
    impl_mode: hand-written
    description: "Re-exports the assertion API from the cclab-qc public crate surface."
```
