---
id: mamba-py312-p0-spec
main_spec_ref: cclab-mamba/testing/conformance.md
merge_strategy: new
---

# Mamba Py312 P0 Spec

## Overview


**main_spec_ref**: `cclab-mamba/testing/conformance.md`

**fill_sections**: overview, requirements, scenarios, test_plan, changes

This change implements all P0 items for Mamba Python 3.12 conformance (#752, #753, #754, #758).

### Scope

1. **Conformance Test Harness (#752)** — Golden-file based test framework that pre-generates expected CPython 3.12 outputs, checks them into the repo, and compares against mamba output in `cargo test`. No CPython dependency at CI time.

2. **MbValue Arithmetic & Comparison (#753)** — Systematic verification of NaN-boxed MbValue arithmetic, comparison, and truthiness against CPython 3.12, including IEEE 754 edge cases and mixed-type promotion.

3. **Object Model (#754)** — Verify and implement missing object model features: C3 MRO, descriptor protocol, metaclass support, `__slots__`, `super()`, and attribute lookup order.

4. **Builtins Verification (#758)** — Verify all ~50+ Python builtins match CPython 3.12 behavior using the conformance harness.

### Design Decisions

- **Golden files over live CPython**: Pre-generate expected outputs, check into repo. CI needs no CPython. Provide `regen` command for refresh.
- **Rust unit tests**: Call mamba interpreter API directly in `cargo test` — no subprocess for mamba side.
- **Implement missing features**: Don't just mark expected-failure for missing object model features (descriptors, metaclass, `__slots__`). Implement them.
- **Full comparison**: Compare stdout + stderr + exception type for thorough conformance.
- **All builtins in one pass**: Verify all builtins systematically since the harness makes it efficient.
## Requirements


### R1: Conformance Test Harness (#752)

| ID | Requirement | Priority |
|----|------------|----------|
| R1.1 | Golden file directory structure: `tests/conformance/{category}/{test_name}.py` + `.expected` | P0 |
| R1.2 | `cargo test` runner that loads `.py`, runs mamba interpreter API, compares stdout/stderr/exception against `.expected` | P0 |
| R1.3 | `regen` command to regenerate golden files from CPython 3.12 | P0 |
| R1.4 | Expected-failure annotations via `# mamba-xfail: <reason>` comment in test `.py` files | P0 |
| R1.5 | Pass/fail/diff report per test case with clear diff output on failure | P0 |
| R1.6 | Category-based test organization: `arithmetic/`, `comparison/`, `class/`, `builtins/`, etc. | P0 |

### R2: MbValue Arithmetic & Comparison (#753)

| ID | Requirement | Priority |
|----|------------|----------|
| R2.1 | int arithmetic: +, -, *, //, /, %, **, unary - (conformance verified) | P0 |
| R2.2 | float arithmetic: same ops + IEEE 754 edge cases (inf, nan, -0.0) | P0 |
| R2.3 | complex arithmetic: +, -, *, /, abs(), conjugate() | P0 |
| R2.4 | Mixed-type promotion: int↔float, int↔complex, float↔complex | P0 |
| R2.5 | Comparison operators across types: ==, !=, <, >, <=, >= | P0 |
| R2.6 | Truthiness: bool(0), bool(""), bool([]), bool(None), bool(1), etc. | P0 |
| R2.7 | Edge cases: round(), abs(), pow(), divmod() | P0 |

### R3: Object Model (#754)

| ID | Requirement | Priority |
|----|------------|----------|
| R3.1 | Class creation with single and multiple inheritance | P0 |
| R3.2 | C3 MRO linearization matching CPython | P0 |
| R3.3 | Descriptor protocol: __get__, __set__, __delete__ | P0 |
| R3.4 | Properties: @property, @x.setter, @x.deleter | P0 |
| R3.5 | Metaclass: `class Meta(type):`, `__init_subclass__` | P0 |
| R3.6 | `__slots__` support | P0 |
| R3.7 | `super()`: zero-arg and explicit forms | P0 |
| R3.8 | `__new__` vs `__init__` ordering | P0 |
| R3.9 | Attribute lookup order: instance → class → bases → __getattr__ | P0 |

### R4: Builtins Verification (#758)

| ID | Requirement | Priority |
|----|------------|----------|
| R4.1 | Numeric builtins: int(), float(), complex(), round(), abs(), pow(), divmod() | P0 |
| R4.2 | Sequence builtins: len(), range(), sorted(), reversed(), enumerate(), zip(), map(), filter() | P0 |
| R4.3 | String builtins: str(), repr(), format(), chr(), ord(), ascii() | P0 |
| R4.4 | Type builtins: type(), isinstance(), issubclass(), callable(), hasattr(), getattr(), setattr() | P0 |
| R4.5 | I/O builtins: print(), input(), open() | P0 |
| R4.6 | Aggregate builtins: all(), any(), min(), max(), sum() | P0 |
| R4.7 | Iterator builtins: iter(), next() | P0 |
| R4.8 | Introspection builtins: id(), hash(), dir(), vars() | P0 |
| R4.9 | Execution builtins: exec(), eval(), compile() | P0 |
## Scenarios


### S1: Run a conformance test that passes

```
Given a test file `tests/conformance/arithmetic/int_add.py` with content `print(1 + 2)`
And a golden file `tests/conformance/arithmetic/int_add.expected` with content `3\n`
When `cargo test` runs the conformance suite
Then mamba output matches the golden file
And the test reports PASS
```

### S2: Run a conformance test that fails with diff

```
Given a test file `tests/conformance/arithmetic/float_div.py` with content `print(1/3)`
And mamba produces `0.33333333333333337` but golden file has `0.3333333333333333`
When `cargo test` runs the conformance suite
Then the test reports FAIL
And shows a clear diff between expected and actual output
```

### S3: Expected-failure test is skipped gracefully

```
Given a test file with `# mamba-xfail: metaclass not yet implemented`
When `cargo test` runs the conformance suite
Then the test is marked as expected-failure (not counted as FAIL)
And if it unexpectedly passes, it reports XPASS for review
```

### S4: Regenerate golden files

```
Given CPython 3.12 is available on the developer's machine
When the developer runs the `regen` command
Then all `.expected` files are regenerated from CPython 3.12 output
And only changed files show up in git diff
```

### S5: Mixed-type arithmetic conformance

```
Given test files covering int+float, int+complex, float+complex promotion
When conformance tests run
Then all type promotion results match CPython 3.12 exactly
Including edge cases: int + inf, float + nan, complex + 0
```

### S6: C3 MRO diamond inheritance

```
Given a Python snippet with diamond inheritance (D(B, C) where B(A), C(A))
When mamba resolves the MRO
Then the linearization matches CPython 3.12 C3 algorithm
And `D.__mro__` == `(D, B, C, A, object)`
```

### S7: Descriptor protocol lookup order

```
Given a class with data descriptor, instance dict, and non-data descriptor
When attribute access occurs
Then lookup follows CPython order: data descriptor > instance > non-data descriptor > __getattr__
```

### S8: Builtin conformance batch verification

```
Given conformance test files for all ~50+ builtins
When the full conformance suite runs
Then each builtin's behavior matches CPython 3.12
Including error cases (TypeError, ValueError for invalid args)
```
## Diagrams

### Sequence Diagram
<!-- TODO -->

### Flowchart
<!-- TODO -->

### Class Diagram
<!-- TODO -->

### State Diagram
<!-- TODO -->

### ERD
<!-- TODO -->

## API Spec

### OpenAPI 3.1
<!-- TODO -->

### OpenRPC 1.3
<!-- TODO -->

### AsyncAPI 2.6
<!-- TODO -->

### Serverless Workflow 0.8
<!-- TODO -->

## Test Plan


### Harness Tests (R1)

| Test ID | Description | Type |
|---------|------------|------|
| T1.1 | Harness loads `.py` + `.expected` pairs from directory | unit |
| T1.2 | Harness compares stdout match → PASS | unit |
| T1.3 | Harness detects stdout mismatch → FAIL with diff | unit |
| T1.4 | Harness detects stderr/exception mismatch → FAIL | unit |
| T1.5 | `# mamba-xfail` annotation skips test gracefully | unit |
| T1.6 | XPASS detection when xfail test unexpectedly passes | unit |
| T1.7 | `regen` command produces valid `.expected` files from CPython 3.12 | integration |

### Arithmetic Tests (R2)

| Test ID | Description | Type |
|---------|------------|------|
| T2.1 | int ops: +, -, *, //, /, %, **, unary - | conformance |
| T2.2 | float ops + IEEE 754: inf, nan, -0.0 | conformance |
| T2.3 | complex ops: +, -, *, /, abs(), conjugate() | conformance |
| T2.4 | Mixed-type promotion chains | conformance |
| T2.5 | Comparison across types | conformance |
| T2.6 | Truthiness for all falsy/truthy values | conformance |
| T2.7 | round(), abs(), pow(), divmod() edge cases | conformance |

### Object Model Tests (R3)

| Test ID | Description | Type |
|---------|------------|------|
| T3.1 | Single/multiple inheritance class creation | conformance |
| T3.2 | C3 MRO: diamond, deep hierarchy, linearization errors | conformance |
| T3.3 | Data/non-data descriptor __get__/__set__/__delete__ | conformance |
| T3.4 | @property with getter/setter/deleter | conformance |
| T3.5 | Metaclass __new__, __init__, __init_subclass__ | conformance |
| T3.6 | __slots__: memory, attribute restriction, inheritance | conformance |
| T3.7 | super(): zero-arg in methods, explicit super(cls, self) | conformance |
| T3.8 | __new__ before __init__ ordering | conformance |
| T3.9 | Attribute lookup order with all descriptor types | conformance |

### Builtins Tests (R4)

| Test ID | Description | Type |
|---------|------------|------|
| T4.1 | Numeric builtins: normal + edge cases | conformance |
| T4.2 | Sequence builtins: empty, single, large inputs | conformance |
| T4.3 | String builtins: ASCII, Unicode, escape sequences | conformance |
| T4.4 | Type builtins: inheritance chains, callable checks | conformance |
| T4.5 | I/O builtins: print separators, end, file | conformance |
| T4.6 | Aggregate builtins: empty iterables, mixed types | conformance |
| T4.7 | Iterator protocol: StopIteration, custom __iter__ | conformance |
| T4.8 | Error cases: TypeError, ValueError for invalid args | conformance |
## Changes


### New Files

| File | Purpose |
|------|---------|
| `crates/mamba/tests/conformance/mod.rs` | Conformance test harness: golden file loader, runner, diff reporter |
| `crates/mamba/tests/conformance/harness.rs` | Core harness logic: run mamba, compare output, xfail handling |
| `crates/mamba/tests/conformance/regen.rs` | Golden file regeneration from CPython 3.12 |
| `crates/mamba/tests/conformance/fixtures/arithmetic/*.py` | Arithmetic conformance test scripts |
| `crates/mamba/tests/conformance/fixtures/arithmetic/*.expected` | Golden files for arithmetic tests |
| `crates/mamba/tests/conformance/fixtures/comparison/*.py` | Comparison conformance test scripts |
| `crates/mamba/tests/conformance/fixtures/class/*.py` | Object model conformance test scripts |
| `crates/mamba/tests/conformance/fixtures/builtins/*.py` | Builtins conformance test scripts |

### Modified Files

| File | Change |
|------|--------|
| `crates/mamba/src/runtime/value.rs` | Fix arithmetic edge cases (IEEE 754, mixed-type promotion) |
| `crates/mamba/src/runtime/class.rs` | Implement missing: descriptors, metaclass, __slots__ |
| `crates/mamba/src/runtime/mro.rs` | Verify/fix C3 linearization algorithm |
| `crates/mamba/src/runtime/builtins.rs` | Fix builtin behaviors to match CPython 3.12 |
| `crates/mamba/src/runtime/object.rs` | Attribute lookup order, descriptor protocol |
| `crates/mamba/Cargo.toml` | Add dev-dependencies for conformance tests |

### Spec Updates

| Spec | Change |
|------|--------|
| `cclab-mamba/testing/conformance.md` | New spec for conformance test harness |
| `cclab-mamba/runtime/value-and-rc.md` | Add conformance requirements for edge cases |
| `cclab-mamba/runtime/class.md` | Add descriptor, metaclass, __slots__ requirements |
| `cclab-mamba/runtime/builtins.md` | Add systematic verification requirements |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: mamba-py312-p0

**Verdict**: APPROVED

### Summary

All 5 sections filled (overview, requirements, scenarios, test_plan, changes). 4 requirement groups (R1-R4) map 1:1 to issues #752-#758. 8 scenarios cover harness pass/fail/xfail, regen, arithmetic edge cases, MRO, descriptor protocol, and bulk builtins. Test plan has 30 test cases across harness, arithmetic, object model, and builtins. main_spec_ref set to cclab-mamba/testing/conformance.md. Design decisions align with pre-clarification answers (golden files, Rust unit tests, implement missing features, full comparison).

### Checklist

- ✅ All requirements traceable to issues
  - R1→#752, R2→#753, R3→#754, R4→#758
- ✅ Scenarios cover key flows
  - 8 scenarios covering pass/fail/xfail/regen/edge cases
- ✅ Test plan covers all requirements
  - 30 test cases across 4 groups
- ✅ Changes list complete
  - New harness files + modified runtime files + spec updates
- ✅ Design decisions match pre-clarifications
  - Golden files, Rust unit tests, implement missing features, all builtins

### Issues

No issues found.
