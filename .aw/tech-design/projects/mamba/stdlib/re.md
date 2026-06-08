---
id: stdlib-re
title: stdlib re — Regular Expressions
crate: mamba
files:
  - crates/mamba/src/runtime/stdlib/re_mod.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: c964bdaf9
---

# stdlib `re`

Regular expressions over the `regex` Rust crate. The `regex` crate is
RE2-shaped (no backtracking, linear-time), so a small set of CPython
features that depend on backtracking are open gaps:

- Lookbehind / lookahead assertions
- Backreferences (`\1` etc.) inside the pattern
- Named groups via `(?P<name>...)` are partial — capture works,
  named back-references don't

For typical match / search / findall / sub / split usage, parity is
full.

Three load-bearing invariants:

1. **Patterns are compiled per-call, NOT cached** — `re.match(p, s)`
   compiles `p` each invocation. CPython caches; Mamba's open gap.
   Tracked under `re.compile` partial coverage.
2. **`re.match` anchors at start; `re.search` does not** — same as
   CPython.
3. **`re.sub` accepts a callable replacement** — when `repl` is
   callable, called per match with the Match object; result is
   substituted. Today partially supported (callable replacement is
   gap; string replacement works).

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: re-types
types:
  ReMod:        { kind: struct, label: "re_mod.rs" }
  RegexCrate:   { kind: struct, label: "regex (Rust crate)" }
  MatchObject:  { kind: struct, label: "Instance class_name=re.Match (group / start / end / groups)" }
  Pattern:      { kind: struct, label: "Instance class_name=re.Pattern (re.compile result)" }
edges:
  - { from: ReMod,    to: RegexCrate,  kind: references, label: "compile + match" }
  - { from: ReMod,    to: MatchObject, kind: owns,       label: "wrap regex Match" }
  - { from: ReMod,    to: Pattern,     kind: owns }
---
classDiagram
    class ReMod
    class RegexCrate
    class MatchObject
    class Pattern
    ReMod --> RegexCrate : compile + match
    ReMod --> MatchObject : wrap
    ReMod --> Pattern : owns
```

## Function catalog
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "re-catalog"
$defs:
  StdlibFnEntry:
    type: object
    properties:
      python_name:    { type: string }
      mb_fn:          { type: string }
      arity:          { type: integer }
      kwargs:         { type: array, items: { type: string } }
      cpython_parity: { type: string, enum: [full, partial, gap] }
      notes:          { type: string }
    required: [python_name, mb_fn, arity, cpython_parity]
  ReCatalog:
    type: array
    items: { $ref: "#/$defs/StdlibFnEntry" }
    examples:
      - - { python_name: "re.search",   mb_fn: "mb_re_search",   arity: 2, cpython_parity: full,    notes: "first match anywhere" }
        - { python_name: "re.match",    mb_fn: "mb_re_match",    arity: 2, cpython_parity: full,    notes: "match must anchor at start" }
        - { python_name: "re.findall",  mb_fn: "mb_re_findall",  arity: 2, cpython_parity: full,    notes: "all non-overlapping matches as list" }
        - { python_name: "re.sub",      mb_fn: "mb_re_sub",      arity: 3, cpython_parity: partial, notes: "string repl works; callable repl gap" }
        - { python_name: "re.split",    mb_fn: "mb_re_split",    arity: 2, cpython_parity: partial, notes: "no maxsplit kwarg yet" }
        - { python_name: "re.escape",   mb_fn: "mb_re_escape",   arity: 1, cpython_parity: full }
        - { python_name: "re.compile",  mb_fn: "(gap)",          arity: 1, cpython_parity: gap,     notes: "compile + reuse cache not wired today" }
        - { python_name: "re.fullmatch", mb_fn: "(gap)",         arity: 2, cpython_parity: gap }
  PatternFeatures:
    description: "RE2 limits — features the regex crate does not support"
    type: array
    items: { type: string }
    examples:
      - - "lookbehind  (?<=...)  /  (?<!...)"
        - "lookahead   (?=...)   /  (?!...)"
        - "backreferences in pattern  (...)\\1"
        - "named backrefs (?P=name)"
```

## Pattern dispatch logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: re-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "re.X(pattern, args)" }
  compile:      { kind: process,  label: "regex::Regex::new(pattern); on err raise re.error" }
  classify:     { kind: decision, label: "search / match / findall / sub / split?" }
  do_search:    { kind: process,  label: "regex.find(haystack); wrap as Match Instance or None" }
  do_match:     { kind: process,  label: "regex.find(haystack); check span starts at 0" }
  do_findall:   { kind: process,  label: "regex.find_iter; collect group(0) strings" }
  do_sub:       { kind: process,  label: "regex.replace_all(haystack, repl_str)" }
  do_split:     { kind: process,  label: "regex.split(haystack); collect parts" }
  done:         { kind: terminal, label: "return MbValue (Match Instance / list / str)" }
edges:
  - { from: enter,      to: compile }
  - { from: compile,    to: classify }
  - { from: classify,   to: do_search,  label: "search" }
  - { from: classify,   to: do_match,   label: "match" }
  - { from: classify,   to: do_findall, label: "findall" }
  - { from: classify,   to: do_sub,     label: "sub" }
  - { from: classify,   to: do_split,   label: "split" }
  - { from: do_search,  to: done }
  - { from: do_match,   to: done }
  - { from: do_findall, to: done }
  - { from: do_sub,     to: done }
  - { from: do_split,   to: done }
---
flowchart TD
    enter([re.X]) --> compile[regex compile]
    compile --> classify{op?}
    classify -->|search| do_search[Match or None]
    classify -->|match| do_match[anchor start]
    classify -->|findall| do_findall[list of strs]
    classify -->|sub| do_sub[replace_all]
    classify -->|split| do_split[parts]
    do_search --> done([result])
    do_match --> done
    do_findall --> done
    do_sub --> done
    do_split --> done
```

## Acceptance scenarios
<!-- type: overview lang: markdown -->

```mermaid
---
id: re-acceptance
actors:
  - { id: User,    kind: actor }
  - { id: Mamba,   kind: system }
  - { id: Fixture, kind: system }
messages:
  - { from: User,    to: Mamba,   name: "run stdlib/re_search_match.py" }
  - { from: Mamba,   to: Fixture, name: "re.search(r'\\d+', 'abc123def'); re.match(r'\\d+', 'abc123')" }
  - { from: Fixture, to: Mamba,   name: "Match at 3:6; None (match anchors)" }
  - { from: User,    to: Mamba,   name: "run stdlib/re_findall.py" }
  - { from: Mamba,   to: Fixture, name: "re.findall(r'\\w+', 'hello world')" }
  - { from: Fixture, to: Mamba,   name: "['hello', 'world']" }
  - { from: User,    to: Mamba,   name: "run stdlib/re_sub_split.py" }
  - { from: Mamba,   to: Fixture, name: "re.sub(r'\\s+', '-', 'a  b c'); re.split(r'\\s+', 'a b c')" }
  - { from: Fixture, to: Mamba,   name: "'a-b-c'; ['a','b','c']" }
  - { from: User,    to: Mamba,   name: "run stdlib/re_lookbehind_gap.py" }
  - { from: Mamba,   to: Fixture, name: "re.search(r'(?<=foo)bar', 'foobar')" }
  - { from: Fixture, to: Mamba,   name: "re.error: lookbehind not supported (RE2 limit)" }
---
sequenceDiagram
    actor User
    participant Mamba
    participant Fixture
    User->>Mamba: search/match
    Mamba->>Fixture: anchor diff
    Fixture-->>Mamba: ok / None
    User->>Mamba: findall
    Mamba->>Fixture: list strs
    Fixture-->>Mamba: matches
    User->>Mamba: sub/split
    Mamba->>Fixture: ws
    Fixture-->>Mamba: dashes / parts
    User->>Mamba: lookbehind
    Mamba->>Fixture: assertion
    Fixture-->>Mamba: re.error gap
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"
fixtures:
  - id: re_search_match
    name: "stdlib/re_search_match.py"
    paired: "stdlib/re_search_match.expected"
  - id: re_findall
    name: "stdlib/re_findall.py"
    paired: "stdlib/re_findall.expected"
  - id: re_sub_split
    name: "stdlib/re_sub_split.py"
    paired: "stdlib/re_sub_split.expected"
  - id: re_groups
    name: "stdlib/re_groups.py"
    paired: "stdlib/re_groups.expected"
    verifies: ["capture groups via match.group(0..n)"]
  - id: re_escape
    name: "stdlib/re_escape.py"
    paired: "stdlib/re_escape.expected"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/stdlib/re_mod.rs
    action: modify
    impl_mode: hand-written
    description: "search / match / findall / sub / split / escape over the regex crate (RE2-shaped). Hand-written; lookahead/lookbehind/backref are open gaps from RE2 substrate. Phase-1 codegen target — entries are mechanical 1:1 wraps."
```
