# Wave-4 sequencing — ship-now confidence ranking

**Date:** 2026-05-15
**Author:** runner-stdlib (Team B, brute-force-then-standardize)
**Scope:** 4 unshipped stdlib libs surveyed for wave-4 conformance ships.

## TL;DR

| Rank | Lib            | Surface           | Predicted regime               | Mem prediction      | Confidence | LOC est | Risk                       |
|------|----------------|-------------------|--------------------------------|---------------------|------------|---------|----------------------------|
| 1    | urllib.parse   | 21 (7+15-3+6)    | compute / balanced              | 0.9-1.1× PASS       | HIGH       | ~280    | NamedTuple ABI design call |
| 2    | csv            | 16 (4+5+7)       | compute / balanced (state-mc)   | 0.7-1.0× PASS       | MED-HIGH   | ~320    | iterator + StringIO wiring |
| 3    | pickle         | 82 (~70+4+8)     | compute (alloc-bound at scale)  | **0.3-0.5× FAIL (subset B)** | MED  | ~420    | __reduce__ #2100 callback  |
| 4    | xml.etree      | 21 (12 def + 9 class) | compute (alloc-bound)      | **0.15-0.30× FAIL (subset B worst)** | LOW-MED | ~520 | many carve-outs            |

**Recommended ship order:** urllib.parse → csv → pickle → xml.etree (or defer xml to wave-5).

## Rationale

### Rank 1 — urllib.parse (ship now, highest confidence)

- Pure-string compute hot path; no #2100 callbacks; no #2129 operator
  overloads; no #2096 subset A (no large-bytes materialization) or
  subset B (no many-small-objects per call beyond the ParseResult
  6-tuple) risk on realistic URL workloads.
- Only design call: NamedTuple ABI for ParseResult/SplitResult.
  Recommend Instance with dual index + attribute access (matches
  divmod pattern extended).
- **Highest perf-PASS confidence in wave-4.** Pre-classifies as
  team-lead's predicted wave-4 leader; my scout confirms.

### Rank 2 — csv (ship next, medium-high)

- State-machine tokenize is pure compute; iterator protocol is
  established pattern (reuse from re.finditer/zip).
- Subset B borderline only at unrealistic scale (10k+ rows). Realistic
  CSV workloads sit sub-#2096-threshold.
- DictReader/DictWriter/Sniffer can defer to wave-5 without breaking
  the forward ship.
- Main risk: io.StringIO interop + Dialect Instance frozen-attribute
  semantics.

### Rank 3 — pickle (ship third, medium)

- Pre-classified subset B per memory; expected mem FAIL by-design
  per #2096. Documented carve-out, ship anyway (precedent: array
  Task #35, json Task #29).
- Bytecode dispatch table is mechanical (~70 opcode constants + 4
  free fns + 6 class shells).
- **Hard carve-out: __reduce__ / __reduce_ex__ are #2100 callback
  territory.** Forward ship of builtin types only (int/float/str/
  bytes/list/dict/tuple/None/bool/frozenset/set).
- Protocol 4 only; protocols 0/1/2/3 + PickleBuffer + out-of-band
  buffers + persistent_id deferred.
- Risk: opcode emitter correctness on edge cases (long ints, NaN
  floats, recursive structures) — needs conformance testing against
  CPython output bytes.

### Rank 4 — xml.etree (defer or ship minimal)

- Pre-classified subset B worst-case (many-elements) per memory.
  Expected mem 0.15-0.30× FAIL — worst in wave-4.
- Largest LOC budget (~520) with the most carve-outs (XPath subset,
  callbacks, namespaces, iterparse, custom XMLParser target).
- **Recommend deferring full ship to wave-5** and shipping only
  `Element` + `fromstring` + `tostring` basic forms in wave-4 if
  budget allows after the other three lands.
- Risk: the carve-out list is long enough that a half-broken surface
  is arguably worse than no surface.

## Cross-cutting confirmations from `phase2_crosscutting_blockers`

- #2096 subset B is **expected and pre-categorized** for pickle and
  xml. Both will mem-FAIL by-design. Ship under the (B-selective)
  carve-out framing.
- #2097 module-attr lookup hoisting applies to all four — bench
  fixtures must hoist.
- #2100 callback-bound applies to pickle's `__reduce__` and xml's
  XMLParser-target callbacks. Both are carved out of the forward
  ship.
- #2128 tuple/frozenset atomic-only-gc-track fast-path benefits
  urllib.parse's NamedTuple returns (str-only contents are
  atomic-eligible).
- #2129 operator-overload gap does not apply to any of the four
  (none define new arithmetic types).

## Ship #1 commitment

Proceeding directly into **Ship #1 = urllib.parse** per team-lead's
"don't wait for approval, go straight in" directive. Will commit
the scout docs first, then begin the urllib.parse implementation
on `project-mamba` directly.

Commit message pattern:
- Scout: `chore(mamba/scout): wave-4 typeshed surface for pickle/csv/xml/urllib.parse`
- Ship: `feat(mamba/urllib_parse): wire <N>-entry surface (#1265 Task #<N>)`

NO Co-Authored-By trailer.
