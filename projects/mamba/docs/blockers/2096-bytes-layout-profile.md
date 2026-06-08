# #2096 subset A — bytes layout profile (Task #57 Phase 1)

- **Date**: 2026-05-15
- **Post-bundle binary**: `015b820a2` (release, M-series)
- **Scope**: Subset A only (bulk-bytes header overhead). Subset B
  (small-object-many-times — json/pickle/csv) deferred per team-lead
  Task #57 brief.

## Phase 1a — Post-#2100 re-bench

The #2100 GC bundle (1A threshold 700 → 10000, 2A FxHash, 3A reduce
fast path) potentially alleviated the iteration-retention amplifier on
subset A by deferring sweeps. Re-bench results:

| Fixture | Pre-#2100 mem | Post-#2100 mem | Δ | Gate (subset-A target ≥0.5×/0.8×/0.3×) |
|---|---|---|---|---|
| array/typed_bulk (iters=10) | 0.09× | **0.26×** | 2.9× | FAIL (target 0.3×) |
| codecs/utf8_bulk (iters=100) | 0.22× | **0.30×** | 1.4× | FAIL (target 0.5×) |
| base64/encode_decode (iters=2000) | 0.53× | **0.53×** | nochange | FAIL (target 0.8×) |

Wall + internal sanity (none regressed):
- array: wall 10.28× PASS, internal 0.25× FAIL (in #2178 cohort)
- codecs: wall 10.05× PASS, internal 1.51× PASS
- base64: wall 14.63× PASS, internal UNAVAIL (fixture has no marker)

**Verdict**: subset A is NOT moot post-#2100. Array improved 3× but
still under 0.3× floor. Codecs improved 1.4× but still under 0.5×
floor. Base64 unchanged. Bytes-layout fix is still required.

## Phase 1b — Structural diagnosis

Direct `std::mem::size_of` probe (transient `_probe_2096_sizeof` test
in `runtime/rc.rs`, reverted before commit):

```
sizeof MbObjectHeader = 8       (AtomicU32 + ObjKind, aligned)
sizeof ObjData        = 96      (enum tag + max-variant padding)
sizeof MbObject       = 104     (header + data)
sizeof Vec<u8>        = 24      (ptr + len + cap)
alignof MbObject      = 8
```

CPython `PyBytesObject` (3.12, x86_64):

```
sizeof PyBytesObject  = 33      (8 ob_refcnt + 8 ob_type + 8 ob_size
                                 + 8 ob_shash + 1 ob_sval[0] start)
                                — followed inline by `len` payload bytes,
                                  single allocation
```

### Per-bytes-object cost comparison

| Bytes size | Mamba header | Mamba allocs | CPython header | CPython allocs | Mamba/CPython ratio |
|---|---|---|---|---|---|
| 16 B | 104 + 24 = **128 B** | 2 (MbObject + Vec buf) | 33 B inline | 1 | **~3.9× per-object** |
| 1 KiB | 104 + 24 = 128 B | 2 | 33 B | 1 | **~1.12×** (header amortized) |
| 1 MiB | 104 + 24 = 128 B | 2 | 33 B | 1 | **~1.0001×** (header negligible) |

`ObjData = 96 bytes` is **dominated by the `CodeObject` and `Instance`
variants**, not by `Bytes`. The `Bytes(Vec<u8>)` variant only needs
24 bytes payload + 8-byte tag = 32 bytes — but every `MbObject` pays
the full 96-byte enum floor.

### Why each fixture mem-fails

- **array/typed_bulk** (1 × 4 MiB bytes/iter × 10 iters held):
  per-object header is negligible (~0.003% of payload). The 0.26× mem
  ratio is **iteration-retention amplifier** — 1A bump deferred GC
  sweep, so peak heap holds multiple 4 MiB buffers simultaneously.
  Pre-#2100 (threshold=700, sweep every ~70 allocs) cleared older bufs
  faster; post-#2100 holds them longer. Net: 1A trades mem for wall.

- **codecs/utf8_bulk** (1 × ~1 MiB bytes + 1 × ~1 MiB str per iter
  × 100 iters): same iteration-retention amplifier as array. Header
  is negligible at 1 MiB scale (~0.012%). Bytes-layout fix won't help
  this fixture much — the lever is GC scheduling, not header size.

- **base64/encode_decode** (2 × ~1.9 KiB bytes/iter × 2000 iters):
  header is ~6% per-object (104 / 1.9KiB) — non-trivial but doesn't
  explain 0.53× (which implies 1.9× total mem overhead). The
  **double allocation** (MbObject heap alloc + separate Vec buffer
  heap alloc per bytes) is the prime suspect — 8000 short-lived
  allocs/sec hammering jemalloc's small-block freelists with
  fragmentation overhead.

## Phase 1c — Fix surface recommendation

The data **does not support a generic "shrink MbObject header"
strategy**. The 96-byte ObjData floor is dominated by other variants
(Instance, CodeObject), and shrinking it would require splitting
`ObjData` into per-kind structs — high-blast-radius header rewrite.

Specific subset-A-targeted options:

### Option A — `SmallBytes` inline variant (REJECTED for subset A)

Inline payload <32 bytes in MbObject. **Helps subset B (small bytes
many times) but the data here shows subset A's mem cost is NOT from
small bytes** — it's iteration-retention (array/codecs) or
double-allocation overhead (base64). Defer to subset B task.

### Option B — Inline `Vec<u8>` buffer with MbObject (REJECTED here)

Single allocation `Box<MbObject + len + cap + payload>` CPython-style.
Removes the second alloc per bytes object. Would help base64's
"double allocation" 1.9× overhead — but touches every bytes-aware
codepath (rc.rs, gc.rs, bytes_ops.rs, every stdlib using
`new_bytes`). **Highest-leverage but ~300-500 LOC blast radius with
JIT lowering implications. Out of subset-A scope per team-lead's
2h Phase 1 budget.**

### Option C — Bytes pool / arena for short-lived bytes (REJECTED here)

Helps base64's freelist-fragmentation case. But the per-iter buffers
are 1-2 KiB — already on jemalloc's small-block fast path; arena
adds bookkeeping that may not beat current fast path.

### **Option D (NEW) — Tune GC for subset-A workloads**

The data shows array/codecs mem regression is **caused by 1A
threshold bump** (#2100 phase 1 trade-off). The fix-surface that
actually addresses subset A is:

1. **Lower threshold for `Bytes` allocations specifically** — every
   `new_bytes` bumps a separate counter; sweep more aggressively when
   bytes-class allocations exceed (e.g.) 50 outstanding.
2. **OR** make `mb_release` on a `Bytes` object eagerly free the
   Vec<u8> buffer without waiting for GC sweep — bytes are
   non-cycle-capable, so RC drop-to-zero is definitive.

Option D2 is **simpler and lower-risk**: ~30 LOC in `rc.rs::mb_release`
to short-circuit `ObjData::Bytes` cleanup. Since `Bytes(Vec<u8>)`
contains no MbValue pointers, no cycle is possible, no GC tracking
is required — drop the Vec<u8> and the MbObject Box immediately on
rc=0.

### Recommended Phase 2 attack

Start with **Option D2** (eager bytes free on rc=0). Expected impact:
- array/typed_bulk (10 large bytes held briefly per iter, rc drops to
  0 at next iter's `a = make_array("i")`) — should flip to ≥0.3×
- codecs/utf8_bulk (bytes+str rebind every iter, rc=0 immediately) —
  should flip to ≥0.5×
- base64/encode_decode (rebind every iter, rc=0 immediately) —
  should flip to ≥0.8×

If D2 alone doesn't flip all three, layer Option B (inline Vec buffer)
for base64's double-alloc cost.

## Acceptance criteria recheck

Per team-lead's Task #57 brief:
- codecs mem ≥ 0.5× (from 0.22×, currently 0.30×) — needs +0.20
- base64 mem ≥ 0.8× (from 0.53×, currently 0.53×) — needs +0.27
- array mem ≥ 0.3× (from 0.09×, currently 0.26×) — needs +0.04 (closest)
- gzip/zlib/lzma/bz2 no regression (not re-benched in Phase 1 yet)
- math/scalar_sqrt canary stable at 3.27× internal (verified
  3.41× in [[project-mamba-2100-gc-bound-resolution]] sweep)

## Open questions for team-lead

1. **Subset A re-scope**: data shows the cost is iteration-retention
   + double-alloc, NOT header overhead. Should we re-scope #2096
   subset A from "bytes header layout" to "bytes lifecycle (eager
   free + double-alloc)"? Or keep the layout framing and pursue
   Option B?

2. **Option D2 risk**: short-circuiting `mb_release` for `Bytes` is
   safe (no cycles) but introduces a code-path divergence from other
   non-cycle types (Str, Tuple-of-atoms). Should we generalize the
   eager-free fast path to all non-cycle-capable types in this task,
   or scope it to Bytes only?

3. **Subset B deferral**: I noticed json/pickle/csv would benefit
   from `SmallBytes` (Option A) inlining since they create many
   small bytes. Filing `2096-subset-B-deferred-notes.md` as
   instructed if the team-lead confirms subset B stays separate.

## References

- [#2096](https://github.com/chrischeng-c4/cclab/issues/2096) — bytes
  2× memory regression (parent issue)
- [#2100](https://github.com/chrischeng-c4/cclab/issues/2100) (closed)
  — GC-bound resolution; 1A threshold bump caused part of subset A
  mem regression
- [#2178](https://github.com/chrischeng-c4/cclab/issues/2178) (open)
  — per-iter JIT alloc; array/typed_bulk's internal-0.25× cohort
  member
- [2100-post-bundle-rebench.md](2100-post-bundle-rebench.md) — full
  sweep table
- `runtime/rc.rs:126-136` — `MbObjectHeader` + `MbObject` defs
- `runtime/rc.rs:147-175` — `ObjData` enum (96-byte floor)
- `runtime/rc.rs:293-299` — `MbObject::new_bytes` (double alloc site)
