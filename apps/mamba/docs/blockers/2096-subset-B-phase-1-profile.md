# #2096 subset B — Phase 1 profile (Task #73)

- **Date**: 2026-05-15
- **Binary**: `d258f1013` + Wave-6 ipaddress at `3d520435b`, release + debuginfo,
  M-series (strip=false override of workspace `[profile.release]`)
- **Scope**: subset B — per-call Instance / Tuple header amortization
  in high-iter / small-payload workloads. Sibling to Phase 1 subset-A
  profile at `84cda7aae` (`2096-bytes-layout-profile.md`).
- **Method**: samply CPU sampler at 4 kHz × 2 M iters of urlparse_bulk
  (~9 600 main-thread samples). Symbolicated via the binary's debuginfo
  + samply `--unstable-presymbolicate` sidecar.

## Headline — bench numbers

| Fixture (cross_runtime_3p) | wall | internal | mem | iters |
|---|---|---|---|---|
| urllib_parse/urlparse_bulk (Instance return) | **5.60× PASS** | **1.07× PASS** | **0.09× FAIL** | 100 000 |
| mimetypes/guess_type_bulk (Tuple return) | **3.26× PASS** | **1.21× PASS** | **0.17× FAIL** | 10×10 000 |

Wall + internal PASS on both fixtures. Mem fails hard on both — urlparse
mamba 131.16 MB vs CPython 12.08 MB; guess_type 78.55 MB vs 13.39 MB.
Same FAIL shape as **D2′ left on codecs+base64** (wall+internal PASS,
mem FAIL), but on Instance / Tuple output instead of Bytes output.
Subset-B resident family.

The Tuple-return variant (guess_type) is **less severe** (0.17× vs 0.09×)
because the Tuple shape allocates fewer per-call MbObjects — only 1
Tuple + 2 Str (or 1 Str + 1 None) = 3 vs urlparse's 7. The
header×N math is consistent: ~3× fewer headers → ~2× better mem ratio.

## Top-3 SELF-time leaves (profile of 2M-iter urlparse)

| % | Function | Lib |
|---|---|---|
| **14.16%** | `nanov2_find_block_and_allocate` | libsystem_malloc.dylib |
| **11.28%** | `nanov2_allocate_outlined` | libsystem_malloc.dylib |
| **9.37%** | `nanov2_malloc` | libsystem_malloc.dylib |

Cumulative system-allocator self-time: **~42%**. Adding adjacent
allocator frames (`_malloc_zone_malloc` 7.44%, `_nanov2_free` 6.16%,
`_platform_memset` 3.67%, `set_tiny_meta_header_in_use` 1.26%) →
**~53% of all CPU time spent in allocator/memset paths**.

The dominant per-call cost is small-block allocator churn — **not**
JIT codegen, **not** op-dispatch overhead, **not** GC sweep. Subset-B
header×N hypothesis CONFIRMED at the profile level.

## Top-30 INCLUSIVE-time frames (call-stack share)

| % | Function |
|---|---|
| 99.96% | `mamba::main` |
| 98.51% | `mamba::driver::CompilerSession::execute_jit_entry` |
| 86.61% | `mamba::runtime::class::mb_call1_val` |
| 84.64% | `mamba::runtime::stdlib::http_mod::dispatch_urlparse` |
| 84.50% | `mamba::runtime::stdlib::http_mod::mb_urllib_urlparse` |
| **60.24%** | `mamba::runtime::stdlib::http_mod::make_parse_result` |
| 25.36% | `hashbrown::map::HashMap::insert` |
| 15.52% | `hashbrown::raw::RawTable::reserve_rehash` |
| 12.81% | `core::hash::BuildHasher::hash_one` |
| 9.43% | `mamba::runtime::class::mb_getattr` |
| 8.01% | `hashbrown::raw::RawTableInner::fallible_with_capacity` |
| 6.98% | `<core::hash::sip::Hasher as core::hash::Hasher>::write` |

**60.24% of all CPU is inside `make_parse_result`** —
the function that builds the per-call `ParseResult` Instance with 6 str
fields (`scheme/netloc/path/params/query/fragment`).

## Per-call alloc accounting (urlparse_bulk hot path)

`make_parse_result` (http_mod.rs:685-707) performs the following per
call:

1. `HashMap::new()` — 1 alloc (initial cap 0)
2. 6× `fields.insert(String, MbValue)` where:
   - Each `String` key is itself constructed via `.into()` from a `&str` literal
     — 6 short-string allocations (≤9 bytes each, fits jemalloc tiny)
   - Each `MbValue` payload calls `MbObject::new_str(scheme)` etc. — 6
     allocations: 1 `Box<MbObject>` (104 B header) + 1 `String` payload
     (≤32 B for hostname/path slices)
   - HashMap repeatedly grows (cap 0 → 4 → 8 likely) → 2-3 rehashes per call
3. `Box::new(MbObject { ... Instance { fields: RwLock::new(HashMap), class_name }})`
   — 1 alloc (104 B header) + the RwLock-wrapped HashMap from step 1
4. `Box::into_raw` returns the pointer; the Instance is **not gc-tracked**
   (the bypass at line 696-706 inlines the construction, skipping
   `MbObject::new_instance`'s `gc_track` call at rc.rs:382)

**Allocs per call**: ~13–14 (1 Instance Box + 6 Str Boxes + 6 String
payloads + 1 HashMap backing storage + 1-2 rehashed backings).

**Header overhead per call**: 7 × 104 B (1 Instance + 6 Str) = **728 B
of MbObject header alone**, before HashMap/payload/RwLock overhead.

**At 100 000 iters**: 7 × 100 000 × 104 B = **~70 MB header floor**
(matches order-of-magnitude of observed 131 MB mamba peak). CPython
`ParseResult` is a `NamedTuple` subclass — ~64 B tuple shell + 6 ×
~33 B PyUnicode = **~262 B per call → ~26 MB at 100k iters**. The 5×
header ratio (728 / 262 ≈ 2.78) compounds against jemalloc's small-block
fragmentation overhead to produce the observed 0.09× mem ratio.

## What the profile RULES OUT

- **GC sweep cost**: no `gc_collect` / `gc_track` frames in top-30 inclusive.
  Instance bypass at make_parse_result:696-706 means these objects aren't
  tracked at all — GC sweep is uninvolved.
- **JIT codegen cost**: cranelift frames absent from steady-state samples
  (likely paid once at startup, ~minor).
- **String interpolation / format!**: no `core::fmt` cluster in top-30.
- **`extract_str` clone**: extract_str (input-side) is post-D2′-equivalent
  already on this shim; no clone churn visible.
- **JIT call/iter alloc** (the #2178 phase 2 cohort): not visible here —
  this fixture's hot loop is in `mb_urllib_urlparse` itself, not
  in the JIT for-loop iter step. urlparse_bulk is NOT in the #2178
  residency.

## What the profile CONFIRMS

- Subset-B mechanism (per-iter MbObject header × N) is real and dominant
  for Instance-shaped per-call returns.
- The cost is **layered**: 104B header is the floor, but the HashMap
  construction (insert + reserve_rehash) **doubles or triples** the
  per-call work above the bare header.
- **mb_getattr at 9.43% inclusive** confirms that even attribute access
  on the returned Instance (the `r.path` in the fixture's `acc += len(r.path)`)
  is non-trivial — it goes through the class.rs dispatch + HashMap
  read.

## Fix-surface candidates (ranked by reversibility × payoff)

### F1 — `new_instance_with_capacity` adoption (cheapest)

`MbObject::new_instance_with_capacity(class_name, 6)` already exists
(rc.rs:389). `make_parse_result` and every similar per-call-Instance
shim (SplitResult, mimetypes' guess_type tuple-wrapper, etc.) should
call it instead of inline-constructing the Instance. Eliminates the
2-3 rehashes per call — the `reserve_rehash` 15.52% inclusive drops to
0%.

**Blast radius**: low — touches each shim that returns a multi-field
Instance, ~20 LOC each. Identified candidates beyond urlparse_bulk:
`SplitResult` (urllib_parse), `(mtype, _enc)` (mimetypes — currently
Tuple, not Instance, but pattern shared), `match` objects (re), `Element`
(xml.etree), `Row` (csv DictReader). ~5-8 callsites.

**Expected impact**: ~15% wall improvement (drop the 15.52% rehash
inclusive), partial mem improvement (allocator churn drops, but the
102 B Instance header × N floor remains).

### F2 — FxHash for Instance HashMap (medium)

The `BuildHasher::hash_one` 12.81% inclusive + `SipHasher::write` 6.98%
self is per-key SipHash13. Switching the Instance fields HashMap from
default `RandomState` to `FxHashMap` (already in tree post-#2100, see
`015b820a2`) trades hash quality for speed.

**Blast radius**: medium — touches the Instance's `fields` type signature
at rc.rs:159 (the ObjData::Instance variant). Every `mb_getattr` /
`mb_setattr` codepath in class.rs that touches `fields` would compile
unchanged (HashMap API is preserved). LOC: ~50.

**Expected impact**: ~12% wall improvement (hash_one cost drops 2-3×).

### F3 — Instance with `Vec<(String, MbValue)>` for small field counts

For ≤8 fields (covers virtually every multi-field stdlib Instance),
replace the HashMap with a linear-scan Vec. Hits the HashMap+rehash+hash
total ~36% inclusive (25.36 + 15.52 - overlap) — but introduces an
ObjData enum split or a bool flag, plus mb_getattr/setattr need to
branch on small-vec vs hashmap layout.

**Blast radius**: high — every Instance-aware codepath needs the
branch. LOC: ~200-300. Risky to ship without careful design.

**Expected impact**: ~25-30% wall improvement + meaningful mem
improvement (no HashMap backing alloc).

### F4 — Inline Instance fields in MbObject (highest leverage, highest risk)

For known-shape stdlib Instances (ParseResult, SplitResult, …), generate
a per-shape variant of `ObjData` with inlined `[MbValue; N]` fields
(named-tuple flavor). Removes the HashMap entirely; pays only the
104 B MbObject header.

**Blast radius**: extreme — requires either codegen per shape or a
runtime variant family. Out of scope for any single-task fix.

### F5 — bench-gate reframe (sibling of subset-A C3)

Document subset-B residency: this fixture class (per-call
multi-field Instance returns at high iter count) is structurally
bounded by `(MbObject_header × num_inner_objects) × iters` against
CPython's NamedTuple+inline shape. The mem-FAIL is a runtime-layout
issue, not a lib-shim issue. Recognize this in bench-gate
authoring rather than chasing per-lib fixes.

## Recommended Phase 2 attack

**Sequence**: F1 → F2 → measure → decide F3/F5.

- **F1 alone** is low-risk pre-commit testable; ~15% wall expected;
  small mem improvement. **Worth shipping first** as a "quick win"
  that also benefits every other Instance-returning shim across
  stdlib (a sibling pattern to D2′'s borrowed-helper approach, but
  on the output side).
- **F2 on top of F1** adds another ~10% wall and 1-2% mem.
- **If F1 + F2 together don't move mem above 0.3× floor**, the
  remaining gap IS structural (header × N) and **F3 or F5 is the
  only path forward**. F3 is the technically clean answer; F5 is the
  cheap audit-trail-only answer.

This sequencing **matches the subset-A Phase 2 sequencing** (D2 → D2′
→ carve-doc), which is the validated playbook from `071ab582c` +
`a62326659`.

## Cross-cutting implications

### Informs #2181 (array handle-leak GC) design

`new_instance_with_capacity` and the bypass-at-make_parse_result
pattern is the same shape as #2181's array handle-table — both rely
on side-channel storage that escapes GC tracking. Any "Instance drop
hook" design for #2181 must consider both:

1. `MbObject::new_instance(class_name)` — gc_tracked path (rc.rs:382)
2. Inline `Box::new(MbObject { … Instance })` bypass paths (used by
   `make_parse_result` and likely others)

If a drop hook lives in `mb_release` for Instance kind, both paths
flow through it. But if it lives in `gc_collect`, the bypass paths
are invisible. Recommend: drop hook in `mb_release`, not in GC.

### Does NOT inform #2178 phase 2 (residual JIT alloc)

urlparse_bulk is **not** in the #2178 internal-<1.0× cohort (its
internal is 1.07× PASS). The residual per-iter JIT alloc that wedges
array/typed_bulk, csv/reader_rows, colorsys, etc. lives in a different
codepath — likely the for-loop iter step itself. This profile
doesn't surface it.

## Acceptance / hand-off

- **Top-3 hot leaves**: nanov2 malloc family — confirms subset-B
  (per-call header×N × allocator churn) over any JIT-codegen /
  dispatch hypothesis
- **Refines dichotomy**: subset B is not monolithic — the
  HashMap-backed Instance variant pays a *layered* cost
  (header + per-key insert/rehash + hash_one) on top of the bare
  104 B header. F1 + F2 target the layered cost cheaply.
- **Hand-off to team-lead**: this doc + the urlparse profile
  artifact (`/tmp/urlparse_profile_sym2.json.gz` + sidecar; not
  committed) provide the evidence for branching the next dispatch
  between:
  - F1/F2 ship (low-risk Phase 2 for subset-B Instance class)
  - #2181 implementation (Instance drop hook, design now informed)
  - #2178 phase 2 (orthogonal — needs its own profile pass)

## References

- [#2096](https://github.com/chrischeng-c4/cclab/issues/2096) — bytes / mem regression parent
- [#2178](https://github.com/chrischeng-c4/cclab/issues/2178) — orthogonal: per-iter JIT alloc cohort
- [#2181](https://github.com/chrischeng-c4/cclab/issues/2181) — array handle-leak (Instance drop hook informed by §"Cross-cutting implications")
- [2096-bytes-layout-profile.md](2096-bytes-layout-profile.md) — sibling subset-A Phase 1
- [2096-codecs-base64-mem-payload-bound.md](2096-codecs-base64-mem-payload-bound.md) — subset-B carve-doc (Bytes-output flavor)
- [2096-subset-B-deferred-notes.md](2096-subset-B-deferred-notes.md) — earlier deferral stub; superseded by this Phase 1 doc
- `runtime/rc.rs:373-400` — `MbObject::new_instance` + `new_instance_with_capacity`
- `runtime/stdlib/http_mod.rs:685-707` — `make_parse_result` hot path
- `runtime/class.rs:1145` — `mb_getattr` (attribute-access leg)
