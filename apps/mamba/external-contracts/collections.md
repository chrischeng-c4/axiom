# collections — external contract (as-is, 2026-07-15)

Domain map: `tech-design/collections/ARCHITECTURE.md` (EC surface section). Verdict law: `HARNESS.md`.
Oracle = live python3.12 byte-diff; xfail = acknowledged gap, still contract (skip, never executed).
`DictKey` Python-hash-domain hazard is mechanically rooted here but the parity write-up lives at
`tech-design/object-model/identity-and-keys.md` §Domain 2 — not restated.

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

No per-type `core/{list,dict,set,tuple}` fixture lib dir exists (ARCHITECTURE.md EC surface); coverage is
concept-bucketed under `tests/cpython/behavior/core/`. Live-counted 2026-07-15 (`ls <dir> | grep -c '\.py$'` +
`grep -l "# mamba-xfail"` per dir):

| Dir | .py | xfail | Covers |
|---|---|---|---|
| `behavior/core/compare/` | 16 | 0 | rich comparison semantics — sequences/mappings/sets/numbers/objects, `__ne__` default-vs-override priority, id-comparison |
| `behavior/core/dictcomps/` | 10 | 4 | dict-literal-via-comprehension construction + scope isolation |
| `behavior/core/listcomps/` | 58 | 57 | list-literal-via-comprehension construction — 98% skipped, 1 live |
| `behavior/core/setcomps/` | 1 | 0 | set-literal-via-comprehension (exception-location smoke only) |
| `behavior/core/container_float_roundtrip/` | 13 | 0 | float-as-dict-key / float-in-container roundtrip (integral-float→int key collapse, list/tuple/set float members) |
| `behavior/core/value_equality_inference/` | 15 | 0 | container `==`/`!=` across list/dict/tuple/set/str/bytes, nested |
| `behavior/core/iter/` | 2 | 2 | iterator free-after-iterating + exception-location — 100% skipped |
| **Total** | **115** | **63** | **52 live** |

Boundary note: `_regression/core/comprehension_scope/` (comprehension scope-leak semantics, not construction) is a
*different* directory owned by `name-resolution`, not this domain — do not conflate with `dictcomps/listcomps/
setcomps` above.

ARCHITECTURE.md's EC surface line additionally names "`errors/core` and `real_world`/`3rd-libs`" as coverage —
neither resolves to a collections-owned directory; see Known contract gaps.

## Negative contract — what must be REJECTED

This domain owns two `type/core/` wall dirs (mamba's opt-in static-annotation enforcement; CPython ignores the
same annotations at runtime, so these walls are mamba-only strictness, not CPython parity):

| Dir | .py | xfail | Rejects |
|---|---|---|---|
| `type/core/container_element/` | 2 | 0 | `dict[str,int]`/`list[int]`-annotated var assigned a wrong-typed element → `TypeError` |
| `type/core/operator_dispatch/` | 3 | 0 | unsupported container binary op (`list - list`, `int + str`, `str * str`) via `eval()` → `TypeError` |
| **Total** | **5** | **0** | all 5 guarded live |

All other `type/` hits with container-adjacent names belong elsewhere: `type/core/method_resolution` and
`type/core/param_types` etc. are object-model/function-domain walls, not this domain's. Any compile reject in a
non-`type/` collections dir above is a type-system false positive by the README.md dimension rule, not this
domain's fault.

## Known contract gaps

- **DictKey raw-`&str` probe hazard**: `IndexMap<DictKey,_>::get(<&str>)` compiles (an `Equivalent<DictKey>` impl
  exists) but hashes in Rust's native `str` domain, landing in the wrong bucket and silently returning `None` for
  present keys — confirmed independently 4x+ before a ~20-site audit swept more call sites (dict_ops.rs:487).
  Mandatory fix shape: probe only via `dict_get_exact_str`/`BorrowedDictStrKey`. tracked: #1566.
- **`{1:_, True:_}` non-collapse**: CPython's `Bool` shares `Int`'s hash/eq domain (`hash(True)==hash(1)`,
  `True==1`), so the two keys collapse to one dict entry; mamba's `DictKey::Bool` is hashed via
  `discriminant+bool` with no `(Int, Bool)` arm in `PartialEq` (dict_ops.rs:748-761: falls to the catch-all
  `_ => false`), so the two keys stay distinct. Genuinely new finding — not covered by #1566 (that hazard is the
  raw-probe API misuse, not this key-identity gap); no existing tracker fits, stating as plain knowledge.
- **`DictKey::Other` NaN-box-identity fallback** (dict_ops.rs:1083): a non-hashable heap object that reaches
  `to_dict_key` without a matching `ObjData` arm is keyed by its NaN-boxed bits formatted as a string instead of
  routing through `__hash__`/`__eq__` — wrong-bucket risk for any future type that lands here without an explicit
  arm. No fixture currently proves this path (no existing tracker confirmed as covering it); plain knowledge.
- **Frozenset membership is O(n)**: `mb_set_contains`'s `ObjData::FrozenSet` arm is a linear `items.iter().any(eq_py)`
  scan (set_ops.rs:170-172) — no hash index, unlike `MbSet`'s `buckets: FxHashMap`. Correctness is unaffected;
  flagged as the "obvious unfilled O(n)→O(1) slot" in ARCHITECTURE.md's Extension points. No tracker; plain
  knowledge (perf-shape, not correctness).
- **`del lst[a:b]` — fixture claim likely stale, needs re-verification**: `mb_list_delitem` implements
  slice-shaped deletion for the 3-part `(start,stop,step)` tuple form (list_ops.rs:1014+) via
  `slice_obj_as_tuple` normalization; the xfail marker on `name-resolution`'s
  `_regression/core/scope_modifiers/del_slice_raises.py` predates commit `900fc7187` (2026-06-15), which appears
  to have already fixed slice-delete through `mb_obj_delitem`/`mb_list_delitem`, but the fixture itself was last
  touched 2026-06-08 and has not been re-run since. Cross-ref: `external-contracts/name-resolution.md` Known
  contract gaps. No GitHub issue was ever filed for this (the in-fixture "(#5)" is an internal item index inside
  the `project_mamba_module_exec_del_silent_divergences` finding set, not a tracker). Re-run the fixture against
  a current build before treating this as an open gap; if it now passes, drop the xfail marker and fold the
  cleanup into #1768 rather than restating it here.
- **C2 perf-pin gap**: zero of the `config/perf/pins/*.toml` pins are bare `list`/`dict`/`set`/`tuple` hot-path
  pins — all are library-level (`collections_1449.toml`, `protobuf_1513.toml`, …) and only exercise dict reads
  indirectly. mamba's hottest primitives ship with no C2 speed/memory guard. No tracker found; plain knowledge
  (restated from ARCHITECTURE.md EC surface, which already flags it).
- **EC-map path drift**: ARCHITECTURE.md's EC surface line names "`errors/core`" and "`real_world`/`3rd-libs`" as
  additional coverage. Neither holds up: `errors/core/`'s three real dirs (`bigaddrspace/`, `flufl/`, `global/`)
  contain no list/dict/set/tuple-specific fixtures; `real_world/` has no `3rd-libs` subdirectory at all (only
  `_regression/3rd-libs/` exists, and it's third-party-package conformance, not collections-owned). The only
  actual `real_world` presence is the broad `real_world/core/cpython321_core_lang/` smoke corpus (391 files spanning
  the whole language), which incidentally touches containers (`lang_dict_merge.py`, `lang_list_methods.py`,
  `lang_tuple_methods.py`, `lang_typeerror_unhashable.py`, …) but is not a dedicated collections bucket. Same
  drift class as object-model.md's `_regression/core/descriptors/` finding. No existing GitHub issue covers this
  specific path-drift (#1771 is unrelated — runner-verdict sidecar/vacuous-walk scope); new finding, not filed.
- **`behavior/core/iter/` fully skipped**: 2/2 xfail, both the generic `auto-ported CPython test; mamba promotion
  pending` marker — 0% live proof that free-after-iterating semantics hold. tracked: #1768.
- **`behavior/core/listcomps/` 98% skipped**: 57/58 xfail under the same auto-ported marker — construction-via-
  comprehension is almost entirely unexercised; only 1 fixture proves anything. tracked: #1768.

## Verification

```bash
# inner loop (seconds; runner-parity verdicts; set MAMBA_BIN after a release build) — from
# apps/mamba/tests/harness/cpython/ (sweep.py resolves relative paths against tests/cpython/ already,
# so args must NOT be prefixed with tests/cpython/ — doing so double-nests and hard-fails)
python3 tools/sweep.py behavior/core/compare \
  behavior/core/dictcomps behavior/core/listcomps \
  behavior/core/setcomps behavior/core/container_float_roundtrip \
  behavior/core/value_equality_inference behavior/core/iter
python3 tools/sweep.py type/core/container_element \
  type/core/operator_dispatch          # negative/wall slice
# cargo slice of the full gate (datatest filter is a path substring; one dir per filter run)
cargo test -p mamba --release --test conformance -- core/compare
cargo test -p mamba --release --test conformance -- core/dictcomps
cargo test -p mamba --release --test conformance -- core/container_element
# C2: none — no bare list/dict/set/tuple perf pin exists (see Known contract gaps); nearest indirect
# coverage is tests/harness/cpython/config/perf/pins/{collections_1449,protobuf_1513}.toml (library-level)
# full gate (the only progress signal): ~3 min; per-fix evidence = before/after readings
cargo test -p mamba --release --test conformance
```

Manifests: none exist for these dirs — hand-authored/pre-manifest, same as object-model's core dirs.
