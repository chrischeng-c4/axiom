# CPython Conformance Fixture Layout

> Canonical spec for the `tests/cpython/fixtures/` tree consumed by the Rust
> Cargo test harnesses under `tests/harness/cpython/`.
> This is the **mamba-Python instantiation** of the repo-wide authoring
> principle in the root `CONTRIBUTING.md` (small · regular · scriptable). Read
> that first for the *why*; this file pins the *how* for Python fixtures: the
> `[tool.mamba]` record, the dimension subdirs, the generate → fill → lint loop,
> and the in-flight migration off the old monolith shape.

## One case = one file

Every fixture is a standalone Python script that exercises **exactly one
concern** and exits `0` under CPython 3.12. No shared helpers, no test classes,
no cross-file state — the subprocess boundary is the isolation guarantee, and a
one-line failure points at one concern. Maximal atomicity is *correct* here: a
fixture is a row in a database, and one-file-per-case is the most scriptable
shape (do not aggregate cases or use in-file case tables).

The **filename is the `case` key** (the linter enforces it). A reader — human or
agent — must tell what a fixture covers from its path alone, without opening it:
a short snake_case phrase naming the single concern, e.g.
`b64encode_roundtrip.py`, `decode_rejects_odd_padding.py`, `isleap_rule.py`.

## The `[tool.mamba]` record

Every fixture embeds a `[tool.mamba]` table **inside** its PEP 723
`# /// script` block. This is the machine-readable record `fixture_gen.py`
writes and `fixture_lint.py` reads (via `tomllib`). It is what makes the tree a
queryable database.

| key            | req? | values |
|----------------|------|--------|
| `bucket`       | yes  | `core` · `builtin-libs` · `std-libs` · `pep` · `type-strict` · `3rd-libs` |
| `lib`          | yes  | the module/topic, e.g. `calendar` (dotted submodules use `_`: `xml_etree`) |
| `dimension`    | yes  | `surface` · `behavior` · `errors` · `bench` · `real_world` · `security` |
| `case`         | yes  | snake_case; **MUST equal the filename stem** |
| `subject`      | yes  | the API under test, e.g. `calendar.isleap` |
| `kind`         | yes  | `mechanical` \| `semantic` |
| `xfail`        | no   | reason string if a known mamba gap (`conformance` skips mamba); `""` = expected pass |
| `mem_carveout` | no   | bench only: opt-out reason; `""` = memory-gated |
| `source`       | no   | CPython oracle provenance, e.g. `Lib/test/test_calendar.py` |
| `status`       | no   | `generated` \| `filled` — semantic files are born `generated`, become `filled` once an agent writes the body; mechanical files are written `filled` |

The block is plain TOML, each line prefixed `# ` (a blank line inside is a bare
`#`). It round-trips through the same extraction the linter uses (strip the
`# `/`#` prefixes, `tomllib.loads`). `xfail` / `mem_carveout` live **in this
table**, not as loose comments (a legacy comment form is still read during
migration — see *Inline directives*).

## Dimension subdirs

For lib-shaped buckets (`std-libs`, `3rd-libs`, `pep`) a `<lib>/` dir holds
**dimension subdirs**, each a flat bag of one-case files:

| Subdir         | Gate         | One case = | kind |
|----------------|--------------|------------|------|
| `surface/`     | `surface`    | one API-existence probe (import / symbol resolves) | mechanical |
| `behavior/`    | `behavior`   | one observable behavior (output matches CPython) | semantic |
| `errors/`      | `errors`     | one exception path (CPython raises; mamba must too) | mechanical |
| `bench/`       | `bench`      | one perf/memory scenario (wall + RSS gated) | semantic |
| `real_world/`  | `real_world` | one end-user integration scenario | semantic |
| `security/`    | `security`   | one adversarial / untrusted-input case | semantic |

```text
fixtures/std-libs/<lib>/
├── surface/<case>.py
├── behavior/<case>.py
├── errors/<case>.py
├── bench/<case>.py
├── real_world/<case>.py
└── security/<case>.py
```

`surface`, `behavior`, `errors`, `real_world`, and `security` share **one
verdict path**: positive fixture, exit `0`, stdout matches CPython (the
MISSING_RAISE promotion, crash ratchet, and type-strict logic apply unchanged).
Only `bench` runs the wall-time + peak-RSS path.

## CPython replacement contract

The fixture tree is a product gate, not only a regression bucket. A mamba build
is allowed to replace CPython for a covered slice only when every axis below is
represented and enforced:

| Axis | Evidence in `tests/cpython` |
|------|------------------------------|
| 100% compatibility, positive path | `surface`, `behavior`, and `real_world` fixtures exit 0 and match the CPython oracle. |
| 100% compatibility, negative path | `errors` fixtures assert CPython's exception taxonomy; a missing raise is a failure. |
| Strong typing | `type-strict` fixtures use inverse markers: CPython may print `no_typeerror:`, but mamba must print/raise `typeerror:`. |
| Faster than CPython | every perf pin sets `floor = 1.0`; `perf_baseline.py record` stores CPython internal time + CPU time in SQLite, then `perf_pin` requires mamba ratios `<= 1.0`. |
| Lower peak memory than CPython | every perf pin sets `mem_floor = 1.0`; the SQLite baseline stores CPython peak RSS, then `perf_pin` requires `cpython_rss / mamba_rss >= 1.0`. |
| Stability and security | `security` fixtures plus `core/compiler_resilience` hostile-source fixtures must raise cleanly or xfail with a tracker, never crash/hang the harness. |

`cargo test -p mamba --test conformance_contract` enforces the structure above.
It is intentionally a meta-test: it checks that the replacement contract cannot
silently lose an axis while the concrete Cargo runners (`conformance`,
`conformance_real_world`, and `perf_pin`) execute the fixtures.

Perf baseline workflow:

```bash
cargo test -p mamba --test cpython_status
cargo test -p mamba --test cpython_status -- --json
python3 tests/harness/cpython/tools/perf_baseline.py record
python3 tests/harness/cpython/tools/perf_baseline.py record --missing-only --ready-only --limit 10 --keep-going
python3 tests/harness/cpython/tools/perf_baseline.py get --pin tests/harness/cpython/config/perf/pins/string_concat_1382.toml
MAMBA_REQUIRE_CPYTHON_PERF_BASELINE=1 cargo test -p mamba --release --test perf_pin -- string_concat_1382
```

The default DB is `tests/cpython/.cache/perf/cpython_baseline.sqlite`. Override
it with `MAMBA_CPYTHON_PERF_BASELINE_DB=/path/to/cpython_baseline.sqlite`.
`cpython_status` is the fast preflight: it reads all pin TOMLs, checks whether
the sqlite DB has a row for each pin, compares fixture hashes for stale rows,
and reports missing CPU/RSS cells plus missing Python prereq imports. Use it to
choose the next `perf_baseline.py record --pin ...` rather than running the
whole perf suite blindly. Its `baseline_recordable_missing_rows` value is the
number of missing rows that can be recorded on the current machine without
installing more Python packages or fixing fixture paths. For normal
development, prefer
`record --missing-only --ready-only --limit <N> --keep-going`: it records only
absent rows, skips pins whose third-party prereqs are not installed on the
current Python, and keeps each baseline batch bounded.

CPython authoring workflow:

```bash
python3 tests/harness/cpython/tools/verify_cpython_oracle.py --bucket std-libs --jobs 8
python3 tests/harness/cpython/tools/verify_cpython_oracle.py --ready-only --jobs 8 --progress-every 1000
python3 tests/harness/cpython/tools/verify_cpython_oracle.py --python tests/cpython/.cache/venv/bin/python --jobs 8
```

This gate is intentionally CPython-only: it proves positive runtime fixtures
exit `0` under CPython and match any `.expected` / `.cpython.expected` golden
before mamba uses them as an oracle. It does not persist pass/fail results.
`bench` fixtures are skipped and owned by `perf_baseline.py`; `# RUN:` pipeline
fixtures are skipped and owned by `tests/harness/cpython/`. The default mode is
strict and treats missing third-party imports as fixture failures. Use
`--ready-only` for the normal local preflight: it skips only third-party
fixtures whose import prereqs are unavailable on the selected Python, so agents
can prove every locally runnable CPython oracle fixture is clean before deciding
whether to install more packages.

CPython source-suite inventory is a separate reference denominator:

```bash
python3 tools/cpython_regrtest_inventory.py
python3 tools/cpython_regrtest_inventory.py --json --top 50
```

Use it to compare CPython `test_*` case candidates against mamba's
one-case-per-file fixtures grouped by dimension before claiming suite-level
coverage. File and module counts are navigation hints only. The numbers are not
identical by design: mamba fixtures are atomic standalone contracts, while
CPython tests include unittest methods, generated variants, doctests,
platform/resource gates, and helpers.

## The canonical one-case file

Self-contained, CPython-3.12 green, deterministic, English, ends in a labelled
`print(...)`. The header carries the PEP 723 block **and** the `[tool.mamba]`
record:

```python
# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "behavior"
# case = "isleap_rule"
# subject = "calendar.isleap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.isleap: Gregorian leap-year rule across div4/common/div400/century."""
import calendar

for year, expected in [(2024, True), (2023, False), (2000, True), (1900, False)]:
    assert calendar.isleap(year) is expected, (year, expected)
print("isleap_rule OK")
```

Hard constraints (full list in `REAL-WORLD-CONVENTION.md`):

- **Standalone & deterministic.** No network, no filesystem writes outside a
  `tempfile.TemporaryDirectory()`, no system mutation, no unseeded `random`, no
  wall-clock comparisons. Same input → same output. No `test.support` /
  `os_helper` / `TESTFN`.
- **CPython-3.12 exit-0.** If the script does not exit `0` under `python3.12`,
  the fixture is broken (`INVALID`) — the oracle's word is final.
- **English only.** Identifiers, comments, docstrings, printed strings.

## Generate → fill → lint

The **script owns structure**; the **agent owns only semantic bodies**.

```
manifest.toml ──fixture_gen.py──▶ skeleton/complete .py ──agent fill──▶ fixture_lint.py
   (the spec)                       (header, [tool.mamba],              (schema + path
                                     imports, mechanical                 + status check)
                                     bodies; placeholder
                                     for semantic bodies)
```

1. **Author a manifest** at `tests/harness/cpython/config/manifests/<bucket>/<lib>.toml`
   — one `[[case]]` per fixture (schema below).
2. **Generate** — `python3.12 tools/fixture_gen.py <manifest.toml>` (or `--all`).
   - **Mechanical** cases (`surface` probes, `errors` `call`/`expect_exc`) are
     emitted **complete** and CPython-green, `status = "filled"`.
   - **Semantic** cases (`behavior` / `security` / `real_world`, or any
     `kind = "semantic"`) are emitted as **skeletons**: the inferred import +
     `# >>> AGENT-FILL: <intent>` + `raise SystemExit("UNFILLED: <case>")` + the
     final `print(...)`, `status = "generated"`.
3. **Fill** — an agent opens *only* the `status = "generated"` skeletons,
   replaces the AGENT-FILL/UNFILLED region with the real body, and flips
   `status` to `"filled"`. The generator is **idempotent**: it never overwrites
   a `filled` file, may regenerate a `generated` one, and creates new ones.
4. **Lint** — `python3.12 tools/fixture_lint.py` enforces the schema, the
   path↔metadata agreement, and that no `filled`/mechanical file still carries a
   placeholder. It **reports** (does not fail on) `status = "generated"`
   skeletons and **LEGACY** (un-migrated) files. Filters: `--bucket`, `--lib`,
   `--unfilled`, `--legacy`; `--strict` fails on legacy too (post-migration).

### Manifest schema

```toml
bucket = "std-libs"
lib = "calendar"

[[case]]                 # mechanical surface probe
dimension = "surface"
case = "isleap_is_callable"
subject = "calendar.isleap"
kind = "mechanical"
probe = "callable"        # callable | not_callable | attr(+attr=) | type(+typename=) | sequence(+length=)
expr = "calendar.isleap"   # optional Python expression to probe; subject remains the coverage key

[[case]]                 # mechanical errors path
dimension = "errors"
case = "month_13_raises"
subject = "calendar.month"
kind = "mechanical"
call = "calendar.month(2024, 13)"
expect_exc = "calendar.IllegalMonthError"

[[case]]                 # semantic behavior (generator emits a skeleton; an agent fills it)
dimension = "behavior"
case = "isleap_rule"
subject = "calendar.isleap"
kind = "semantic"
intent = "Gregorian leap-year rule across div4/common/div400/century"
source = "Lib/test/test_calendar.py"
```

## Worked cases

- **surface (mechanical)** — `calendar/surface/isleap_is_callable.py`: manifest
  `probe = "callable"` → body `assert callable(calendar.isleap)`.
- **errors (mechanical)** — `calendar/errors/month_13_raises.py`: manifest
  `call`/`expect_exc` → a `try/except` that prints `type(e).__name__`.
- **behavior (semantic)** — `calendar/behavior/isleap_rule.py`: the canonical
  template above; the agent wrote the loop body and set `status = "filled"`.
- **security (semantic)** — `zipfile/security/decompression_bomb_ratio_guard.py`:
  `subject = "zipfile.ZipFile"`, `intent` names the attack + guard.
- **known gap (xfail)** — `struct/errors/pack_value_out_of_range_raises.py`:
  set `xfail = "struct shim truncates instead of raising (WI #3929)"` in the
  manifest; the `conformance` harness then skips mamba for it.

## Gate assignment (DUAL-SHAPE)

The Rust harnesses under `tests/harness/cpython/` assign each `.py` under a lib
dir to exactly one gate, in order:

1. `<lib>/<dim>/**.py` where `<dim>` is a dimension subdir → that gate (the
   dimension subdir is the **first** path component under the lib dir).
2. A legacy root monolith: `surface.py` → `surface`, `behavior.py` →
   `behavior`, `errors.py` → `errors`.
3. Anything else → the `feature` catch-all (same verdict path as `behavior`), so
   **nothing is silently dropped** mid-migration.

### Retired-but-supported monoliths

The old shape was one monolith per gate at the lib root. These are **retired**
but still **discovered** — migration is incremental, so both shapes are
supported simultaneously and may coexist in one lib. Migrate by splitting each
monolith into one-case files under the matching dimension subdir (author a
manifest, generate, fill), then deleting the monolith.

## Feature-shaped buckets

`core`, `builtin-libs`, and `type-strict` are NOT lib-shaped: every `.py` under
a lib dir is the `feature` gate (flat, recursive). They are unaffected by the
dimension-subdir model (the `[tool.mamba]` record still applies as they migrate).

## Inline directives (legacy form, still read)

Before the `[tool.mamba]` record, gating was carried by head-of-file comments;
the Rust harnesses still read them during migration:

- `# mamba-xfail: <reason>` — equivalent to `[tool.mamba].xfail`. Short-circuits
  before mamba runs so a known failure never reddens CI; cite a tracker/memory.
- `# mamba-mem-carveout: <reason>` — equivalent to `[tool.mamba].mem_carveout`.
  Bench-gate only; peak RSS is still measured and reported as `[CARVE_OUT: …]`.

New fixtures use the `[tool.mamba]` fields; the comment form is the legacy/dual
path only.

## Adding a fixture (checklist)

1. Add a `[[case]]` to `config/manifests/<bucket>/<lib>.toml` (pick dimension,
   `case`, `subject`, `kind`).
2. `python3.12 tools/fixture_gen.py <manifest>` — emits the file (mechanical
   complete; semantic = skeleton).
3. If semantic, fill the `AGENT-FILL` body and flip `status` to `filled`.
4. `python3.12 tools/fixture_lint.py --lib <lib>` → schema-clean, 0 unfilled.
5. `python3.12 tools/verify_cpython_oracle.py --bucket <bucket> --lib <lib>`
   exits `0` for the CPython side.
6. `mamba run <file>` matches, or set `xfail` in the manifest.
7. The Cargo harness picks it up automatically — no runner edit needed.
