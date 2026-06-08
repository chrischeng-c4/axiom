---
id: mamba-py312-conformance-spec
main_spec_ref: "crates/mamba/testing/mamba-py312-conformance.md"
merge_strategy: update
filled_sections: [overview, requirements, scenarios, cli, test_plan, changes]
fill_sections: [overview, requirements, scenarios, cli, test_plan, changes]
---

# Mamba Py312 Conformance Spec

## Overview

<!-- type: overview lang: markdown -->

Extends Mamba's conformance test suite to full Python 3.12 behavioral parity (#1037). Every builtin function (108), every data structure method, all 81 implemented stdlib modules, and all core language features must produce output identical to CPython 3.12.

**Infrastructure**: Reuses existing golden-file harness (`conformance_tests.rs` + `regen_golden.py`). Python fixtures in `tests/fixtures/conformance/{category}/` run through the Mamba JIT pipeline; stdout is compared against `.expected` golden files pre-generated from CPython 3.12. No live CPython needed at test time.

**Coverage expansion**:

| Category | Current Fixtures | Target |
|----------|-----------------|--------|
| Builtins | 7 partial files | All 108 functions across categorized fixtures |
| Data Structures | 8 files (list, dict, set, str, tuple partial) | Complete: list×33, dict×17, set×17, str×47, bytes/bytearray all methods |
| Stdlib | 0 conformance fixtures | All 81 implemented modules |
| Language Features | generators, exceptions, decorators (partial) | Class MRO/descriptors/metaclass, pattern matching (PEP 634), comprehensions (PEP 709 scoping), async generators |
| CLI | not present | `cclab mamba test --conformance` subcommand |

**Zero-divergence policy**: All divergences from CPython 3.12 must be fixed within this change. `# mamba-xfail: <reason>` is only permitted for genuinely unimplemented features with no planned scope (e.g. `except*` PEP 654, `asyncio` event loop internals). Every xfail must reference an open GitHub issue.

**Fixture structure**: `tests/fixtures/conformance/{category}/{feature}.py` + `{feature}.expected`. New categories: `class_system/`, `stdlib/{module}/`, `builtins/{group}/`, `language/`.
## Requirements

<!-- type: requirements lang: markdown -->

### R1: Complete Builtin Coverage

| ID | Requirement | Priority |
|----|------------|----------|
| R1.1 | Numeric builtins: `abs`, `divmod`, `pow`, `round`, `sum`, `min`, `max` — all edge cases (negative, float, overflow) | P1 |
| R1.2 | Type conversion: `int`, `float`, `bool`, `str`, `bytes`, `bytearray`, `chr`, `ord`, `hex`, `oct`, `bin` | P1 |
| R1.3 | Sequence builtins: `len`, `range`, `enumerate`, `zip`, `reversed`, `sorted`, `filter`, `map` — all argument forms | P1 |
| R1.4 | Collection constructors: `list`, `tuple`, `set`, `frozenset`, `dict` — from iterables, keyword args, empty | P1 |
| R1.5 | Introspection: `type`, `isinstance`, `issubclass`, `id`, `hash`, `repr`, `dir`, `vars`, `callable` | P1 |
| R1.6 | Object protocol: `getattr`, `setattr`, `delattr`, `hasattr`, `object` | P1 |
| R1.7 | Iteration utilities: `iter`, `next`, `all`, `any` — exhaustion, short-circuit, StopIteration | P1 |
| R1.8 | I/O builtins: `print` (sep, end, flush), `input`, `open`, `format` | P1 |
| R1.9 | Functional: `staticmethod`, `classmethod`, `property`, `super` | P1 |
| R1.10 | Metaclass: `__build_class__`, `__import__` | P2 |

### R2: Complete Data Structure Method Coverage

| ID | Requirement | Priority |
|----|------------|----------|
| R2.1 | `list` — all 33 methods: append, clear, copy, count, extend, index, insert, pop, remove, reverse, sort + operators + slicing + comparison | P1 |
| R2.2 | `dict` — all 17 methods: clear, copy, fromkeys, get, items, keys, pop, popitem, setdefault, update, values + merge (`\|`, `\|=`) | P1 |
| R2.3 | `set`/`frozenset` — all 17 methods: add, clear, copy, discard, difference, intersection, isdisjoint, issubset, issuperset, pop, remove, symmetric_difference, union + operators | P1 |
| R2.4 | `str` — all 47 methods: capitalize, casefold, center, count, encode, endswith, expandtabs, find, format, format_map, index, isalnum, isalpha, isascii, isdecimal, isdigit, isidentifier, islower, isnumeric, isprintable, isspace, istitle, isupper, join, ljust, lower, lstrip, maketrans, partition, removeprefix, removesuffix, replace, rfind, rindex, rjust, rpartition, rsplit, rstrip, split, splitlines, startswith, strip, swapcase, title, translate, upper, zfill | P1 |
| R2.5 | `bytes`/`bytearray` — all methods: decode, fromhex, hex, split, strip, replace, find, startswith, endswith + mutable bytearray ops | P1 |
| R2.6 | `tuple` — immutability, unpacking, `*`-unpacking, count, index, hashing, lexicographic comparison | P1 |

### R3: Stdlib Module Conformance

| ID | Requirement | Priority |
|----|------------|----------|
| R3.1 | Priority stdlib (16): `json`, `os`, `re`, `datetime`, `collections`, `pathlib`, `math`, `sys`, `io`, `csv`, `hashlib`, `itertools`, `functools`, `struct`, `random`, `asyncio` — full public API conformance | P1 |
| R3.2 | Extended stdlib (remaining 65 modules Mamba implements): abc, argparse, array, ast, atexit, base64, bisect, bz2, calendar, cmath, codecs, configparser, contextlib, copy, dataclasses, decimal, difflib, dis, enum, errno, fractions, gc, glob, gzip, heapq, hmac, html.parser, http, importlib, inspect, locale, logging, lzma, math, numbers, operator, pickle, platform, pprint, queue, secrets, shlex, shutil, signal, socket, sqlite3, statistics, string, subprocess, tarfile, tempfile, textwrap, threading, time, tokenize, traceback, tracemalloc, types, typing, unicodedata, unittest, unittest.mock, uuid, warnings, weakref, xml, zipfile, zlib | P2 |
| R3.3 | Each stdlib fixture tests public API surface: construction, core methods, error cases, edge cases. Output matched against CPython 3.12 golden files | P1 |

### R4: Language Feature Conformance

| ID | Requirement | Priority |
|----|------------|----------|
| R4.1 | Class system: single/multiple inheritance, MRO (C3 linearization), `super()`, `__init_subclass__`, `__init__`/`__new__` | P1 |
| R4.2 | Descriptors: `__get__`/`__set__`/`__delete__`, `@property`, `@staticmethod`, `@classmethod` | P1 |
| R4.3 | Metaclass: `type` as metaclass, custom `__metaclass__` (via `type.__new__`) | P2 |
| R4.4 | Pattern matching (PEP 634): literal, capture, sequence, mapping, class, OR, AS, wildcard patterns | P1 |
| R4.5 | Comprehensions (PEP 709): list/dict/set comprehension scope isolation, nested comprehensions, walrus operator in comprehension | P1 |
| R4.6 | Decorators: stacked, parameterized, class decorators, `functools.wraps` | P1 |
| R4.7 | Generator full protocol: `yield`, `yield from`, `send()`, `throw()`, `close()`, `StopIteration.value`, async generators (xfail) | P1 |
| R4.8 | Exception full coverage: `BaseException` tree, `except` subclass matching, `raise from`, `__cause__`/`__context__`/`__traceback__`, `ExceptionGroup`/`except*` (xfail per #755) | P1 |
| R4.9 | Context managers: `__enter__`/`__exit__`, `contextlib.contextmanager`, `with` statement semantics | P1 |
| R4.10 | String interpolation: f-strings (nested, conversion flags `!r`/`!s`/`!a`, format spec), multiline f-strings | P1 |

### R5: Conformance CLI Runner

| ID | Requirement | Priority |
|----|------------|----------|
| R5.1 | `cclab mamba test --conformance` runs all fixtures under `tests/fixtures/conformance/` | P1 |
| R5.2 | Reports pass/fail per fixture with divergence diff when output mismatches golden file | P1 |
| R5.3 | `--category <name>` flag filters to a specific conformance category | P2 |
| R5.4 | `--regen-golden` flag regenerates all `.expected` files from CPython 3.12 (delegates to `regen_golden.py`) | P2 |
| R5.5 | Exit code 0 only when all non-xfail fixtures pass | P1 |

### Constraints

- All existing 40 conformance fixtures must continue to pass
- Total test suite (1745+ tests) must not regress
- Each xfail must have `# mamba-xfail: <reason> (see #<issue>)` and reference an open issue
- Golden files are generated from CPython 3.12.x (latest patch)
- No live CPython dependency at test runtime
## Scenarios

<!-- type: scenarios lang: markdown -->

### S1: Full builtins suite passes

```
Given conformance fixtures for all 108 Python builtins (numeric, type-conversion, sequence, collection, introspection, I/O)
When `cargo test -p mamba --test conformance_tests` runs
Then every builtin fixture output matches its CPython 3.12 golden file
And zero fixtures are unexpectedly xfailed
```

### S2: Complete data structure method conformance

```
Given fixtures for list (33 methods), dict (17), set (17), str (47), bytes/bytearray, tuple
When cargo test runs the conformance suite
Then all method outputs, edge cases, and exception messages match CPython 3.12
```

### S3: Stdlib module conformance — priority 16

```
Given conformance fixtures for json, os, re, datetime, collections, pathlib, math, sys, io, csv, hashlib, itertools, functools, struct, random, asyncio
When cargo test runs
Then each module's public API produces identical output to CPython 3.12
And unimplemented asyncio event-loop internals are marked xfail with issue references
```

### S4: Stdlib module conformance — extended 65 modules

```
Given conformance fixtures for each of the 65 additional implemented stdlib modules
When cargo test runs
Then each module's core API produces identical output to CPython 3.12
And non-conformant behavior is either fixed or explicitly xfailed with issue reference
```

### S5: Class system MRO conformance

```
Given fixtures exercising single inheritance, multiple inheritance (diamond), super(), __init_subclass__, descriptors
When cargo test runs class_system conformance fixtures
Then MRO resolution order matches CPython 3.12 C3 linearization exactly
And descriptor protocol (__get__/__set__/__delete__) behaves identically
```

### S6: Pattern matching full conformance

```
Given fixtures for all 8 PEP 634 pattern types: literal, capture, sequence, mapping, class, OR, AS, wildcard
When cargo test runs language/pattern_matching fixtures
Then every match expression produces same result as CPython 3.12
```

### S7: Divergence detected — fixture fails

```
Given a conformance fixture where Mamba produces different output than CPython 3.12
When cargo test runs
Then the test FAILS (not xfail)
And the failure message shows a unified diff between actual and expected output
And the developer fixes the Mamba runtime bug before this change can merge
```

### S8: Legitimate xfail — unimplemented feature

```
Given a fixture for ExceptionGroup/except* (PEP 654) marked `# mamba-xfail: ExceptionGroup not implemented (see #755)`
When cargo test runs
Then the fixture is skipped with xfail status
And the test suite still exits 0
And removing the xfail marker causes the test to run and fail expectedly
```

### S9: CLI conformance runner

```
Given `cclab mamba test --conformance` is invoked
When the command runs all conformance fixtures via cargo test
Then it prints a summary: total fixtures, passed, failed, xfailed
And exits with code 0 when all non-xfail fixtures pass
And exits with code 1 if any non-xfail fixture fails
```

### S10: Golden file regeneration

```
Given a new conformance fixture .py file with no .expected file
When `python3 tests/regen_golden.py` (or `cclab mamba test --regen-golden`) is run
Then a .expected file is generated from CPython 3.12 stdout for each .py fixture
And subsequent cargo test passes for that fixture
```
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

<!-- type: test-plan lang: markdown -->

### Verification Commands

```bash
# Run full conformance suite
cargo test -p mamba --test conformance_tests

# Run via CLI (after implementing cclab mamba test --conformance)
cclab mamba test --conformance

# Run specific category
cclab mamba test --conformance --category builtins
cclab mamba test --conformance --category stdlib/json

# Regenerate golden files
python3 crates/mamba/tests/regen_golden.py

# Full regression (no new failures)
cargo test -p mamba
```

---

### TC-R1: Builtin Coverage

**TC-R1.1 — Numeric builtins edge cases**

```
Given fixtures/conformance/builtins/numeric.py covering abs, divmod, pow, round, sum, min, max
And golden file numeric.expected generated from CPython 3.12
When cargo test runs conformance_tests::builtins::numeric
Then all outputs match CPython 3.12 exactly
And negative values, floats, and overflow paths are exercised
```

**TC-R1.2 — Type conversion builtins**

```
Given fixtures/conformance/builtins/type_conversions.py covering int, float, bool, str, bytes, bytearray, chr, ord, hex, oct, bin
And golden file type_conversions.expected from CPython 3.12
When cargo test runs conformance_tests::builtins::type_conversions
Then all conversion outputs match CPython 3.12 byte-for-byte
And boundary values (e.g. int("-0"), chr(0), chr(0x10FFFF)) match
```

**TC-R1.3 — Sequence builtins all argument forms**

```
Given fixtures/conformance/builtins/sequence.py covering len, range, enumerate, zip, reversed, sorted, filter, map
And golden file sequence.expected from CPython 3.12
When cargo test runs conformance_tests::builtins::sequence
Then all argument forms (start/stop/step for range, key/reverse for sorted, etc.) produce identical output
And lazy iterators materialised via list() match CPython 3.12
```

**TC-R1.4 — Collection constructors from iterables**

```
Given fixtures/conformance/builtins/collection_builtins.py covering list(), tuple(), set(), frozenset(), dict()
And golden file collection_builtins.expected from CPython 3.12
When cargo test runs conformance_tests::builtins::collection_builtins
Then construction from iterable, keyword args, and empty form all match CPython 3.12
```

**TC-R1.5 — Introspection builtins**

```
Given fixtures/conformance/builtins/type_builtins.py covering type, isinstance, issubclass, id, hash, repr, dir, vars, callable
And golden file type_builtins.expected from CPython 3.12
When cargo test runs conformance_tests::builtins::type_builtins
Then isinstance/issubclass MRO traversal matches CPython 3.12
And repr output for built-in types matches CPython 3.12 format
```

**TC-R1.6 — Object protocol builtins**

```
Given fixtures/conformance/builtins/object_protocol.py covering getattr, setattr, delattr, hasattr, object()
And golden file object_protocol.expected from CPython 3.12
When cargo test runs conformance_tests::builtins::object_protocol
Then AttributeError messages, getattr default fallback, and delattr semantics match CPython 3.12
```

**TC-R1.7 — Iteration utilities exhaustion and short-circuit**

```
Given fixtures/conformance/builtins/iteration.py covering iter, next, all, any
And golden file iteration.expected from CPython 3.12
When cargo test runs conformance_tests::builtins::iteration
Then StopIteration propagation, all([]) == True, any([]) == False match CPython 3.12
And short-circuit behaviour (all/any stopping on first definitive value) matches
```

**TC-R1.8 — I/O builtins**

```
Given fixtures/conformance/builtins/io_builtins.py covering print(sep, end, flush), format, open
And golden file io_builtins.expected from CPython 3.12
When cargo test runs conformance_tests::builtins::io_builtins
Then print with custom sep/end produces identical stdout bytes
And format() with format specs matches CPython 3.12
```

**TC-R1.9 — Functional builtins**

```
Given fixtures/conformance/builtins/functional.py covering staticmethod, classmethod, property, super
And golden file functional.expected from CPython 3.12
When cargo test runs conformance_tests::builtins::functional
Then descriptor protocol for staticmethod/classmethod/property matches CPython 3.12
And super() resolution in single and multiple inheritance matches CPython 3.12
```

---

### TC-R2: Data Structure Method Coverage

**TC-R2.1 — list all 33 methods**

```
Given fixtures/conformance/data_structures/list_methods.py and list_slicing.py
And golden files generated from CPython 3.12
When cargo test runs conformance_tests::data_structures::list_*
Then all 33 list methods produce identical output to CPython 3.12
And slice with step, negative index, and out-of-bounds access matches
And sort stability and comparison operators match CPython 3.12
```

**TC-R2.2 — dict all 17 methods and insertion order**

```
Given fixtures/conformance/data_structures/dict_methods.py and dict_comprehension.py
And golden files from CPython 3.12
When cargo test runs conformance_tests::data_structures::dict_*
Then all 17 dict methods match CPython 3.12 output
And insertion order is preserved in all iteration (keys, values, items)
And merge operator (| and |=) matches CPython 3.12
```

**TC-R2.3 — set/frozenset all 17 methods**

```
Given fixtures/conformance/data_structures/set_ops.py
And golden file from CPython 3.12
When cargo test runs conformance_tests::data_structures::set_ops
Then all 17 set/frozenset methods and operators match CPython 3.12
And frozenset is immutable and hashable as per CPython 3.12
```

**TC-R2.4 — str all 47 methods**

```
Given fixtures/conformance/data_structures/string_methods.py
And golden file from CPython 3.12
When cargo test runs conformance_tests::data_structures::string_methods
Then all 47 str methods produce byte-for-byte identical output to CPython 3.12
And Unicode edge cases (casefold, encode, isalpha on non-ASCII) match
```

**TC-R2.5 — bytes/bytearray all methods**

```
Given fixtures/conformance/data_structures/bytes_ops.py
And golden file from CPython 3.12
When cargo test runs conformance_tests::data_structures::bytes_ops
Then all bytes/bytearray methods match CPython 3.12
And bytearray mutability and in-place operations match CPython 3.12
```

**TC-R2.6 — tuple immutability, unpacking, hashing**

```
Given fixtures/conformance/data_structures/tuple_ops.py
And golden file from CPython 3.12
When cargo test runs conformance_tests::data_structures::tuple_ops
Then tuple immutability (TypeError on assignment), *-unpacking, count, index, and lexicographic comparison match CPython 3.12
And hash(tuple) is deterministic and consistent with CPython 3.12
```

---

### TC-R3: Stdlib Module Conformance

**TC-R3.1 — Priority 16 stdlib modules full API**

```
Given conformance fixtures for: json, os, re, datetime, collections, pathlib, math, sys, io, csv, hashlib, itertools, functools, struct, random, asyncio
And each golden file generated from CPython 3.12 (random uses fixed seed)
When cargo test runs conformance_tests::stdlib::* for each priority module
Then every public API call produces identical output to CPython 3.12
And asyncio event-loop internals are marked xfail with reference to open issue
```

**TC-R3.2 — Extended 65 stdlib modules**

```
Given conformance fixtures for each of the 65 additional implemented stdlib modules
And golden files from CPython 3.12
When cargo test runs the extended stdlib conformance suite
Then each module's core public API produces identical output to CPython 3.12
And any non-conformant behaviour is either fixed or explicitly xfailed with issue reference
```

**TC-R3.3 — Stdlib fixture structure validation**

```
Given any stdlib conformance fixture .py file
When the fixture runs through the Mamba JIT pipeline
Then construction, core methods, error cases, and edge cases are all exercised
And the fixture output is compared against the CPython 3.12 golden file
And no live CPython process is invoked at test time
```

---

### TC-R4: Language Feature Conformance

**TC-R4.1 — Class system MRO (C3 linearization)**

```
Given fixtures/conformance/class_system/inheritance.py with single, multiple, and diamond inheritance
And golden file from CPython 3.12
When cargo test runs conformance_tests::class_system::inheritance
Then MRO resolution order matches CPython 3.12 C3 linearization exactly
And __init_subclass__ hook fires in the correct order
```

**TC-R4.2 — Descriptor protocol**

```
Given fixtures/conformance/class_system/descriptors.py covering __get__/__set__/__delete__, @property
And golden file from CPython 3.12
When cargo test runs conformance_tests::class_system::descriptors
Then data descriptor vs non-data descriptor precedence matches CPython 3.12
And @property getter/setter/deleter behaviour matches CPython 3.12
```

**TC-R4.3 — super() cooperative multiple inheritance**

```
Given fixtures/conformance/class_system/super_call.py covering super() in __init__ and cooperative MI
And golden file from CPython 3.12
When cargo test runs conformance_tests::class_system::super_call
Then super().__init__ call order follows MRO and matches CPython 3.12
```

**TC-R4.4 — Pattern matching all 8 PEP 634 types**

```
Given fixtures/conformance/language/pattern_matching.py with literal, capture, sequence, mapping, class, OR, AS, wildcard patterns
And golden file from CPython 3.12
When cargo test runs conformance_tests::language::pattern_matching
Then every match expression produces the same result as CPython 3.12
And guard conditions and fall-through semantics match CPython 3.12
```

**TC-R4.5 — Comprehension scope isolation (PEP 709)**

```
Given fixtures/conformance/language/comprehension_scope.py with list/dict/set comprehensions and walrus operator
And golden file from CPython 3.12
When cargo test runs conformance_tests::language::comprehension_scope
Then the iteration variable does not leak into the enclosing scope
And walrus operator (:=) in comprehensions assigns to the correct enclosing scope
And nested comprehension scoping matches CPython 3.12
```

**TC-R4.6 — Decorators: stacked, parameterised, class**

```
Given language fixtures for stacked decorators, parameterised decorators, class decorators, functools.wraps
And golden files from CPython 3.12
When cargo test runs conformance_tests::language::decorators
Then decorator application order, __wrapped__ attribute, and __name__/__doc__ preservation match CPython 3.12
```

**TC-R4.7 — Generator full protocol**

```
Given language fixtures for yield, yield from, send(), throw(), close(), StopIteration.value
And golden files from CPython 3.12
When cargo test runs conformance_tests::language::generators
Then generator state machine transitions match CPython 3.12 exactly
And send(value) return from yield matches CPython 3.12
And throw(exc) propagation matches CPython 3.12
And async generator fixtures are marked xfail referencing an open issue
```

**TC-R4.8 — Exception hierarchy and chaining**

```
Given language fixtures for BaseException tree, except subclass matching, raise from, __cause__/__context__/__traceback__
And golden files from CPython 3.12
When cargo test runs conformance_tests::language::exceptions
Then exception matching via isinstance against the MRO matches CPython 3.12
And raise X from Y sets __cause__ and suppresses __context__ as per CPython 3.12
And ExceptionGroup/except* fixtures are marked xfail referencing issue #755
```

**TC-R4.9 — Context managers**

```
Given fixtures/conformance/language/context_managers.py covering with statement, __enter__/__exit__, contextlib.contextmanager
And golden file from CPython 3.12
When cargo test runs conformance_tests::language::context_managers
Then __exit__ receives exception info on error and (None, None, None) on clean exit
And contextlib.contextmanager yield-based CM semantics match CPython 3.12
```

**TC-R4.10 — Advanced f-strings**

```
Given fixtures/conformance/language/fstring_advanced.py with nested f-strings, !r/!s/!a conversion flags, format spec
And golden file from CPython 3.12
When cargo test runs conformance_tests::language::fstring_advanced
Then all f-string forms produce byte-for-byte identical output to CPython 3.12
And multiline f-strings match CPython 3.12
```

---

### TC-R5: Conformance CLI Runner

**TC-R5.1 — CLI runs full conformance suite and exits 0**

```
Given all conformance fixtures are present and all non-xfail fixtures pass
When `cclab mamba test --conformance` is invoked
Then all fixtures under tests/fixtures/conformance/ are executed
And a summary is printed: total fixtures, passed, failed, xfailed counts
And the process exits with code 0
```

**TC-R5.2 — CLI reports divergence diff on failure**

```
Given a conformance fixture where Mamba output diverges from the golden file
When `cclab mamba test --conformance` is invoked
Then the failing fixture is reported by name
And a unified diff between actual and expected output is displayed
And the process exits with code 1
```

**TC-R5.3 — --category flag filters fixtures**

```
Given the builtins conformance category has N fixtures
When `cclab mamba test --conformance --category builtins` is invoked
Then only builtins/* fixtures are executed
And stdlib, language, and class_system fixtures are not run
```

**TC-R5.4 — --regen-golden regenerates expected files**

```
Given a .py conformance fixture with no corresponding .expected file
When `cclab mamba test --regen-golden` is invoked
Then regen_golden.py is executed under CPython 3.12
And a .expected file is created for each .py fixture
And a subsequent cargo test run passes for that fixture
```

**TC-R5.5 — Exit code 0 only when all non-xfail pass**

```
Given a conformance run where all non-xfail fixtures pass and N fixtures are xfailed
When `cclab mamba test --conformance` completes
Then exit code is 0
And xfailed fixtures are listed but do not affect the exit code
```

---

### TC-Zero: Zero-Divergence and Regression Policy

**TC-ZD.1 — No regression in existing 40 conformance fixtures**

```
Given the existing 40 conformance fixtures that currently pass
When the full test suite runs after this change
Then all 40 existing fixtures still pass
And no previously passing test is newly xfailed or failing
```

**TC-ZD.2 — Total suite (1745+) does not regress**

```
Given the full Mamba test suite of 1745+ tests
When `cargo test -p mamba` runs
Then the total pass count does not decrease
And no existing passing test becomes failing or panics
```

**TC-ZD.3 — Every xfail references an open issue**

```
Given any fixture containing the directive `# mamba-xfail:`
When the conformance suite parses xfail directives
Then each xfail directive has the format `# mamba-xfail: <reason> (see #<issue>)`
And the referenced issue number is an open GitHub issue
And no xfail is present for a feature that is fully implemented
```
# Run full conformance suite
cargo test -p mamba --test conformance_tests

# Run via CLI (after implementing cclab mamba test --conformance)
cclab mamba test --conformance

# Run specific category
cclab mamba test --conformance --category builtins
cclab mamba test --conformance --category stdlib/json

# Regenerate golden files
python3 crates/mamba/tests/regen_golden.py

# Full regression (no new failures)
cargo test -p mamba
```

### Builtin Tests

| Test ID | Fixture | Coverage |
|---------|---------|----------|
| T1.1 | `builtins/numeric.py` | abs, divmod, pow, round, sum, min, max — edge cases |
| T1.2 | `builtins/type_conversions.py` | int, float, bool, str, bytes, chr, ord, hex, oct, bin |
| T1.3 | `builtins/sequence.py` | len, range, enumerate, zip, reversed, sorted, filter, map |
| T1.4 | `builtins/collection_builtins.py` | list(), tuple(), set(), frozenset(), dict() constructors |
| T1.5 | `builtins/type_builtins.py` | type, isinstance, issubclass, id, hash, repr, dir, vars, callable |
| T1.6 | `builtins/object_protocol.py` | getattr, setattr, delattr, hasattr, object() |
| T1.7 | `builtins/iteration.py` | iter, next, all, any — exhaustion, short-circuit |
| T1.8 | `builtins/io_builtins.py` | print (sep/end/flush), format, open |
| T1.9 | `builtins/functional.py` | staticmethod, classmethod, property, super |

### Data Structure Tests

| Test ID | Fixture | Coverage |
|---------|---------|----------|
| T2.1 | `data_structures/list_methods.py` | All 33 list methods, operators, comparison |
| T2.2 | `data_structures/list_slicing.py` | Slicing with all forms: step, negative, out-of-bounds |
| T2.3 | `data_structures/dict_methods.py` | All 17 dict methods, view objects, insertion order |
| T2.4 | `data_structures/dict_comprehension.py` | Dict comprehension, merge operator |
| T2.5 | `data_structures/set_ops.py` | All 17 set/frozenset methods and operators |
| T2.6 | `data_structures/string_methods.py` | All 47 str methods, encoding, f-strings |
| T2.7 | `data_structures/tuple_ops.py` | Tuple immutability, unpacking, hashing, comparison |
| T2.8 | `data_structures/bytes_ops.py` | bytes/bytearray all methods, mutable ops |

### Stdlib Tests

| Test ID | Fixture | Coverage |
|---------|---------|----------|
| T3.1 | `stdlib/json/json_encode_decode.py` | json.loads, json.dumps, edge cases |
| T3.2 | `stdlib/re/pattern_matching.py` | re.match, re.search, re.findall, groups |
| T3.3 | `stdlib/datetime/datetime_ops.py` | datetime, date, timedelta, formatting |
| T3.4 | `stdlib/collections/deque_counter.py` | deque, Counter, OrderedDict, defaultdict, namedtuple |
| T3.5 | `stdlib/math/math_ops.py` | All math module functions, special values |
| T3.6 | `stdlib/itertools/itertools_ops.py` | chain, product, permutations, combinations, cycle, groupby |
| T3.7 | `stdlib/functools/functools_ops.py` | partial, reduce, lru_cache, wraps |
| T3.8 | `stdlib/hashlib/hash_ops.py` | md5, sha256, sha512, update/hexdigest |
| T3.9–T3.81 | `stdlib/{module}/*.py` | One fixture per implemented module |

### Language Feature Tests

| Test ID | Fixture | Coverage |
|---------|---------|----------|
| T4.1 | `class_system/inheritance.py` | Single/multiple inheritance, MRO |
| T4.2 | `class_system/descriptors.py` | __get__/__set__/__delete__, property |
| T4.3 | `class_system/super_call.py` | super() in __init__, cooperative multiple inheritance |
| T4.4 | `language/pattern_matching.py` | All 8 PEP 634 pattern types |
| T4.5 | `language/comprehension_scope.py` | PEP 709 scope isolation, nested comprehensions |
| T4.6 | `language/context_managers.py` | with statement, __enter__/__exit__, contextlib |
| T4.7 | `language/fstring_advanced.py` | Nested f-strings, conversion flags, format spec |

### CLI Tests

| Test ID | Verification |
|---------|--------------|
| T5.1 | `cclab mamba test --conformance` exits 0 when all non-xfail fixtures pass |
| T5.2 | `cclab mamba test --conformance --category builtins` runs only builtins fixtures |
| T5.3 | `cclab mamba test --conformance` exits 1 when any non-xfail fixture fails |
| T5.4 | `cclab mamba test --regen-golden` regenerates .expected files from CPython 3.12 |
## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  # ── CLI: mamba test --conformance ─────────────────────────────
  - path: crates/cclab-cli/src/mamba.rs
    action: MODIFY
    desc: Add `test` subcommand with --conformance, --category, --regen-golden flags; delegates to cargo test for conformance suite

  # ── Builtins conformance fixtures ─────────────────────────────
  - path: crates/mamba/tests/fixtures/conformance/builtins/object_protocol.py
    action: CREATE
    desc: getattr/setattr/delattr/hasattr/object() conformance
  - path: crates/mamba/tests/fixtures/conformance/builtins/iteration.py
    action: CREATE
    desc: iter/next/all/any — exhaustion, short-circuit, StopIteration
  - path: crates/mamba/tests/fixtures/conformance/builtins/functional.py
    action: CREATE
    desc: staticmethod/classmethod/property/super conformance
  - path: crates/mamba/tests/fixtures/conformance/builtins/object_protocol.expected
    action: CREATE
    desc: Golden file from CPython 3.12
  - path: crates/mamba/tests/fixtures/conformance/builtins/iteration.expected
    action: CREATE
    desc: Golden file from CPython 3.12
  - path: crates/mamba/tests/fixtures/conformance/builtins/functional.expected
    action: CREATE
    desc: Golden file from CPython 3.12

  # ── Data structures: bytes/bytearray ──────────────────────────
  - path: crates/mamba/tests/fixtures/conformance/data_structures/bytes_ops.py
    action: CREATE
    desc: bytes/bytearray all methods conformance
  - path: crates/mamba/tests/fixtures/conformance/data_structures/bytes_ops.expected
    action: CREATE
    desc: Golden file from CPython 3.12

  # ── Class system fixtures ──────────────────────────────────────
  - path: crates/mamba/tests/fixtures/conformance/class_system/inheritance.py
    action: CREATE
    desc: Single/multiple inheritance, MRO (C3 linearization) conformance
  - path: crates/mamba/tests/fixtures/conformance/class_system/descriptors.py
    action: CREATE
    desc: Descriptor protocol (__get__/__set__/__delete__) and @property conformance
  - path: crates/mamba/tests/fixtures/conformance/class_system/super_call.py
    action: CREATE
    desc: super() in __init__, cooperative multiple inheritance
  - path: crates/mamba/tests/fixtures/conformance/class_system/init_subclass.py
    action: CREATE
    desc: __init_subclass__ hook conformance
  - path: crates/mamba/tests/fixtures/conformance/class_system/inheritance.expected
    action: CREATE
    desc: Golden file from CPython 3.12
  - path: crates/mamba/tests/fixtures/conformance/class_system/descriptors.expected
    action: CREATE
    desc: Golden file from CPython 3.12
  - path: crates/mamba/tests/fixtures/conformance/class_system/super_call.expected
    action: CREATE
    desc: Golden file from CPython 3.12
  - path: crates/mamba/tests/fixtures/conformance/class_system/init_subclass.expected
    action: CREATE
    desc: Golden file from CPython 3.12

  # ── Language feature fixtures ──────────────────────────────────
  - path: crates/mamba/tests/fixtures/conformance/language/pattern_matching.py
    action: CREATE
    desc: All 8 PEP 634 pattern types conformance
  - path: crates/mamba/tests/fixtures/conformance/language/comprehension_scope.py
    action: CREATE
    desc: PEP 709 comprehension scope isolation, nested comprehensions, walrus in comprehension
  - path: crates/mamba/tests/fixtures/conformance/language/context_managers.py
    action: CREATE
    desc: with statement, __enter__/__exit__, contextlib.contextmanager
  - path: crates/mamba/tests/fixtures/conformance/language/fstring_advanced.py
    action: CREATE
    desc: Nested f-strings, conversion flags (!r/!s/!a), format spec
  - path: crates/mamba/tests/fixtures/conformance/language/pattern_matching.expected
    action: CREATE
    desc: Golden file from CPython 3.12
  - path: crates/mamba/tests/fixtures/conformance/language/comprehension_scope.expected
    action: CREATE
    desc: Golden file from CPython 3.12
  - path: crates/mamba/tests/fixtures/conformance/language/context_managers.expected
    action: CREATE
    desc: Golden file from CPython 3.12
  - path: crates/mamba/tests/fixtures/conformance/language/fstring_advanced.expected
    action: CREATE
    desc: Golden file from CPython 3.12

  # ── Stdlib fixtures (one .py + .expected per module) ──────────
  - path: crates/mamba/tests/fixtures/conformance/stdlib/json/json_encode_decode.py
    action: CREATE
    desc: json.loads, json.dumps, nested types, edge cases
  - path: crates/mamba/tests/fixtures/conformance/stdlib/re/pattern_matching.py
    action: CREATE
    desc: re.match, re.search, re.findall, re.sub, named groups
  - path: crates/mamba/tests/fixtures/conformance/stdlib/datetime/datetime_ops.py
    action: CREATE
    desc: datetime, date, time, timedelta, strftime/strptime
  - path: crates/mamba/tests/fixtures/conformance/stdlib/collections/collections_ops.py
    action: CREATE
    desc: deque, Counter, OrderedDict, defaultdict, namedtuple, ChainMap
  - path: crates/mamba/tests/fixtures/conformance/stdlib/math/math_ops.py
    action: CREATE
    desc: All math module functions, inf, nan, pi, e constants
  - path: crates/mamba/tests/fixtures/conformance/stdlib/itertools/itertools_ops.py
    action: CREATE
    desc: chain, product, permutations, combinations, cycle, groupby, islice
  - path: crates/mamba/tests/fixtures/conformance/stdlib/functools/functools_ops.py
    action: CREATE
    desc: partial, reduce, lru_cache, wraps, cached_property
  - path: crates/mamba/tests/fixtures/conformance/stdlib/hashlib/hash_ops.py
    action: CREATE
    desc: md5, sha256, sha512, update/hexdigest/digest
  - path: crates/mamba/tests/fixtures/conformance/stdlib/os/os_ops.py
    action: CREATE
    desc: os.path, os.environ, os.getcwd, os.listdir, os.getpid
  - path: crates/mamba/tests/fixtures/conformance/stdlib/pathlib/pathlib_ops.py
    action: CREATE
    desc: Path construction, navigation, glob, stat
  - path: crates/mamba/tests/fixtures/conformance/stdlib/struct/struct_ops.py
    action: CREATE
    desc: struct.pack, struct.unpack, format strings
  - path: crates/mamba/tests/fixtures/conformance/stdlib/random/random_ops.py
    action: CREATE
    desc: seeded random — randint, choice, shuffle, sample, random (deterministic with seed)
  - path: crates/mamba/tests/fixtures/conformance/stdlib/csv/csv_ops.py
    action: CREATE
    desc: csv.reader, csv.writer, DictReader, DictWriter
  - path: crates/mamba/tests/fixtures/conformance/stdlib/io/io_ops.py
    action: CREATE
    desc: StringIO, BytesIO, read/write/seek/tell
  - path: crates/mamba/tests/fixtures/conformance/stdlib/sys/sys_ops.py
    action: CREATE
    desc: sys.argv, sys.version_info, sys.path, sys.modules keys
  - path: crates/mamba/tests/fixtures/conformance/stdlib/asyncio/asyncio_basics.py
    action: CREATE
    desc: asyncio.run, basic coroutine, gather — event loop internals xfailed
  - path: crates/mamba/tests/fixtures/conformance/stdlib/**/*.expected
    action: CREATE
    desc: Golden files generated from CPython 3.12 for all stdlib fixtures above
  - path: crates/mamba/tests/fixtures/conformance/stdlib/{remaining_65_modules}/*.py
    action: CREATE
    desc: One fixture + golden file per remaining implemented stdlib module (abc, argparse, array, ast, atexit, base64, bisect, ...)

  # ── Runtime bug fixes (discovered during conformance) ─────────
  - path: crates/mamba/src/runtime/
    action: MODIFY
    desc: Fix runtime divergences discovered when running each fixture against CPython 3.12; files TBD per divergence found

  # ── Spec update ───────────────────────────────────────────────
  - path: cclab/specs/crates/mamba/testing/mamba-py312-conformance.md
    action: MODIFY
    desc: Update with full conformance scope (extends P0/P1 spec to cover all builtins, all stdlib, all language features)
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews