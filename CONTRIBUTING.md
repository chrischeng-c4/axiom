# Contributing

## Authoring principle: right-sized files, semantic paths, explicit names

> An agent or human should learn *what exists* and *where to act* from `ls`,
> paths, and filenames alone — without opening files. Every file you don't open
> is a saved tool call and less context burned.

This is a multi-language ecosystem (Rust runtime + libraries, TS/UI, Python
conformance tests, specs, generated code, configs, docs, handoffs). The
principle below is **medium-agnostic** — it is about *navigability*, not about
any language. Make the repository legible from its structure, so an agent can
decide where to act, and tooling can operate on the tree, without opening many
files.

### The three rules

1. **Right-sized files — one coherent concern per file.** A file should have a
   single reason to exist and a single reason to be opened. "Right-sized" is the
   point, not "small": the optimal grain depends on the *access pattern* (do
   readers scan-and-skip, or read many at once?) and on *cohesion* (does the
   content form one concept, or several independent ones?). See *Balanced
   splitting*.
2. **Semantic paths — the directory IS the taxonomy.** The path classifies the
   content and conveys a file's role before you open it. You should be able to
   predict an artifact's path from its identity, and vice versa.
3. **Explicit names — the leaf name briefs the content.** `ls <dir>/` should read
   as a table of contents. A name that needs the file body to understand it is a
   defect — rename it (prefer the concrete observable, `isleap_rule`, over a
   vague `misc_cases`).

### Balanced splitting

Splitting earns its keep only when it improves navigation, reviewability, reuse,
or selective execution. Use the rule both ways.

**Split when** any holds:
- a reader would otherwise search *inside* the file for one independent concern;
- the pieces are owned or reviewed separately;
- the pieces are executed, skipped, generated, or compared independently;
- the resulting leaf names would form a useful table of contents.

**Do _not_ split when** any holds:
- the pieces only make sense read together — high-cohesion logic shredded across
  many files becomes a cross-file puzzle, not a clearer tree;
- the split would create trivial wrapper files;
- a shared setup dominates the content (the setup, not the cases, is the file);
- the directory would just get noisier without improving discovery.

Rule of thumb: if a file needs internal section headers separating *unrelated*
concerns, split it; if its parts share one concept or one setup, keep them. (So:
several independently named/reviewed/executed concerns → consider splitting; a
single rule exercised over a few representative inputs → one file, as a table.)

### Granularity scales with tooling

> Judgment call, called deliberately: the finer you split, the more files must
> stay mutually consistent — so **push granularity as fine as your tooling can
> keep consistent, no finer.**

Where a generator emits the regular structure and a linter enforces it,
consistency is mechanical and maximal one-concern-per-file is cheap and
sustainable (you never hand-maintain the many files). Where files are
hand-written and hand-kept-consistent, that consistency cost bites at scale, so
lean toward cohesion. **Generate + validate ⇒ go fine; hand-author ⇒ stay
coherent.** This is why the Python fixture tree (below) goes maximally atomic —
it is fully tooled — while hand-written source should not.

### Path grammar (a pattern, not a mandate)

```
<area>/<subject>/<concern>/<artifact>
```

- **area** — broad repo area: `tests`, `specs`, `configs`, `generated`, `docs`,
  `handoffs`, …
- **subject** — the module / feature / protocol / package / service / topic.
- **concern** — *the question this file answers, or its role*: behavior, errors,
  security, performance, integration, schema, api, migration, … (an open idea,
  not a fixed list — each tree names its own concerns).
- **artifact** — the specific case / scenario / generated unit / config concern /
  document.

Not every tree needs four levels — use the depth the tree earns. The same
grammar reads across media:

```
configs/auth/oauth_token_lifetime.yaml
specs/http/errors/malformed_header_rejected.md
generated/parser/ast/node_kinds.ts
handoffs/release/2026-05-rc1-risk-summary.md
tests/std-libs/calendar/behavior/isleap_rule.py     # worked below
```

### Mamba test architecture: DDD, boundary-first

`projects/mamba/tests/` is organized by capability domain, not by runner
technology. The tree should explain what boundary a test pins before an agent
opens a file:

```text
projects/mamba/tests/
├── cpython/     CPython replacement contract: runtime parity, strict type
│                deltas, speed, memory, security, and compatibility fixtures.
├── mambalibs/   Mamba-native library contracts with no CPython oracle.
├── pkgmgr/      `mamba` CLI and package-manager behavior.
└── governance/  Meta-gates over manifests, release profiles, CI policy, and
                 test-inventory shape.
```

Use these boundaries when adding tests:

- **`cpython/`** defines what mamba must do to replace CPython for a covered
  slice. It answers: `surface` has the API, `behavior` matches CPython,
  `errors` raises where CPython raises, `real_world` works in user-shaped
  flows, `security` does not crash or hang, and `bench` is faster and
  lower-memory than CPython.
- **`type-strict`** is the deliberate incompatibility lane inside CPython
  conformance: CPython may accept a case, but mamba must reject it with the
  explicit inverse markers. Do not mix these cases into ordinary compatibility
  fixtures.
- **`mambalibs/`** is not CPython compatibility. It covers mamba-only native
  library loading, ABI, generated stubs, and local artifact behavior.
- **`pkgmgr/`** owns package-manager CLI behavior. It should spawn the built
  `mamba` binary and pin observable CLI outcomes.
- **`governance/`** owns contracts about the test system itself: manifest
  schemas, release gates, profile definitions, skip debt, and structural lints.

Cargo only auto-discovers top-level `tests/*.rs`; mamba therefore uses explicit
`[[test]]` entries in `projects/mamba/Cargo.toml`. New Rust integration tests
belong under a domain directory and must be wired through a domain entrypoint
or umbrella runner, not dropped as ad-hoc root files. Domain-local helper
scripts also stay under their domain, for example
`tests/cpython/tools/regen_golden.py` rather than `tests/regen_golden.py`. The
domain root should contain only entrypoints and taxonomy directories; concrete
cases live below the taxonomy. For example, a parse-only Python case belongs in
`tests/cpython/fixtures/core/parse/`, not in `tests/governance/`. The domain
grammar is:

```text
tests/<domain>/<subject>/<concern>/<artifact>
```

For CPython fixtures, the concrete grammar is:

```text
tests/cpython/fixtures/<bucket>/<lib>/<dimension>/<case>.py
```

For manifest-backed governance gates, keep the manifest and the Rust checker
discoverably paired:

```text
tests/governance/gates/<scope>/<gate>/manifest.toml
tests/governance/schema_gates/<scope_or_gate>_fixture_<issue>.rs
```

For mambalibs:

```text
tests/mambalibs/fixtures/<gate>/manifest.toml
tests/mambalibs/mambalibs_<gate>_fixture_<issue>.rs
```

Current migration debt: some CPython fixtures still exist as legacy monoliths
such as `<lib>/surface.py`, `<lib>/behavior.py`, and `<lib>/errors.py`. They are
discovered for backward compatibility, but new fixtures must use the
dimension-directory shape. The `governance/ci_guard.rs` test locks the current
count as a ceiling so the debt can only stay flat or shrink.

### Where it applies (scope)

Strongest for **naturally decomposable** trees — independent test fixtures,
config entries, generated units, doc/handoff files — where each artifact is
genuinely standalone and (ideally) tool-maintained. Applied with **judgment** to
cohesive hand-written code: there, one-concept-one-file can rightly *outweigh*
file count, and a language's idioms win (Rust `#[test]` fns stay in a
`mod tests`; a cohesive module groups related items). This is a guideline for
legible structure, not a mandate to shred cohesive code into wrapper files.

---

## Example instantiation: Python test fixtures

> One worked instantiation of the principle above — **not** the definition. The
> full spec lives in
> `projects/mamba/tests/cpython/conventions/FIXTURE-LAYOUT.md`.

The mamba CPython conformance suite is the reference adopter. Each fixture is one
self-contained case; the path is the grammar made concrete —
`<bucket>/<lib>/<dimension>/<case>.py` — where `dimension` is the *concern* for
tests (one of: surface · behavior · errors · bench · real_world · security). It
goes **maximally atomic** precisely because it is fully tooled — `fixture_gen`
emits the structure from a manifest and `fixture_lint` enforces it — so the
"granularity scales with tooling" rule licenses one-case-per-file here.

### Worked case: decomposing a behavior monolith

**Before** — one file, eight unrelated behaviors mixed together:

```
std-libs/calendar/behavior.py        # 8 cases, one big file
```

**After** — the dimension is a directory; each case is a named leaf:

```
std-libs/calendar/behavior/
  isleap_rule.py                 # leap-year rule (a few representative years, one table)
  leapdays_counts.py
  monthrange_february.py
  setfirstweekday_roundtrip.py
  ...
```

`ls behavior/` is now the spec, and a reader jumps straight to the one case they
need. No coverage is lost — and note `isleap_rule.py` keeps the rule's several
input years *together* as one table (cohesion), rather than one-file-per-input
(which would be over-splitting).

### Python fixture conventions (this tree only)

These apply to `.py` fixtures under the conformance tree — **not** to the general
principle:

- PEP 723 `# /// script` header (+ a `[tool.mamba]` record) so `uv run <file>` is
  hermetic and the tree is queryable.
- CPython-3.12 exit-0 is the oracle; any filesystem use goes through `tempfile`;
  end on one labelled `print(...)`.
- `# mamba-xfail:` / `# mamba-mem-carveout:` directives (or the equivalent
  `[tool.mamba]` fields).
- CPython authoring verification runs through
  `python3 projects/mamba/tests/cpython/tools/verify_cpython_oracle.py`; it
  proves runtime fixtures exit `0` and match any golden, while bench fixtures
  stay owned by the perf baseline flow.
- CPython source-suite sizing is a separate reference denominator. Use
  `python3 projects/mamba/tools/cpython_regrtest_inventory.py` before claiming
  broad CPython coverage; the primary comparison is CPython `test_*` case
  candidates versus mamba one-case-per-file fixtures grouped by dimension.
  File count and module count are only navigation hints.
- `projects/mamba/tests/harness/cpython/` owns discovery, execution,
  collection, and reporting through Cargo test binaries such as
  `conformance`, `conformance_contract`, `cpython_status`, and `perf_pin`.
  `projects/mamba/tests/cpython/` stays the contract-data tree: fixtures,
  config, conventions, and Python helper tools.
- Harness quality is part of the CPython replacement goal: if a harness is
  wrong, hides useful failure data, collects the wrong metrics, or makes normal
  development loops too slow, fix the harness before adding more runtime work.
- Performance gates use a machine-local CPython SQLite baseline generated by
  `python3 projects/mamba/tests/cpython/tools/perf_baseline.py record`; it
  stores internal fixture time, CPU time, and peak RSS under
  `projects/mamba/tests/cpython/.cache/perf/`.
- Before a long perf run, use
  `cargo test -p mamba --test cpython_status -- --json` (or omit `--json` for
  a capped human summary). It reports missing/stale perf baseline rows, missing
  CPU/RSS cells, missing fixture files, and missing Python prereq imports per
  pin. The `baseline_recordable_missing_rows` count is the immediate local
  batch size for pins whose fixtures and Python prereqs are ready.
- Fill baseline gaps incrementally with
  `python3 projects/mamba/tests/cpython/tools/perf_baseline.py record --missing-only --ready-only --limit <N> --keep-going`.
  This records only missing rows whose Python prereqs are available locally, so
  agents can grow the sqlite baseline in small verified batches.

See `FIXTURE-LAYOUT.md` for the six-dimension table, the file template, and the
`fixture_gen` → fill → `fixture_lint` loop.
