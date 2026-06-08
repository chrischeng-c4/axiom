# Mamba Roadmap — Path to Py3.12 Replacement

> Last updated: 2026-04-30 (WS-A.1 complete)
> Owner: `epic-py3-12-single-master-tracking`
> Status: **C1, C4, C5, P4 done. C2 partial. C3 active critical path — WS-A.1 introspection cluster ✅; WS-A.2 pytest is the next P1 step. P1/P2/P3 open.**

This document is the converged view across all open mamba epics + issues
after the 2026-04-30 audit. It supersedes any drift between individual
issue bodies and ground truth on `main`.

---

## North Star
<!-- type: overview lang: markdown -->

**Mamba = Python 3.12 功能等效 + 效能超越.** Single-binary mamba can run
any unmodified Python 3.12 program (sans C-extension fast paths) and
match or beat CPython 3.12 perf on equivalent workloads.

C-extension dependencies (gevent, greenlet, grpc, protobuf C fast path)
are **out of scope** of this roadmap and tracked under separate research.

---

## Where we are
<!-- type: overview lang: markdown -->

```
[Layer 1: Core]      ✅ 583/583 conformance, 54/54 cpython_compat
                        ↓
[Layer 2: Builtins]  🟢 Type methods + most builtins ✅
                        Introspection cluster (globals/locals/vars/dir) ✅ DONE 2026-04-30
                        ↓
[Layer 3: Stdlib]    🟡 ~21/36 modules upgraded to callable dispatchers
                        ~15 modules still stub-only (long-tail tracker)
                        ↓
[Layer 4: 3P libs]   ❌ pytest / Flask / requests not yet running
                        Active critical path (3 sub-issues filed)
```

| Master criterion | State | Evidence / next |
|------------------|-------|-----------------|
| C1 — 5 conformance suites pass | ✅ DONE | 583/583 + 54/54 on `main` |
| C2 — Top 50 stdlib ≥95% surface | 🟡 ~21 done, ~15 long-tail | `enhancement-mamba-stdlib-stub-completion-tracker` |
| C3 — pytest / Flask / requests run unmodified | 🟡 ACTIVE | 3 sub-issues filed; introspection prerequisite ✅ DONE 2026-04-30; pytest is next |
| C4 — Zero P0/P1 runtime bugs | ✅ DONE 2026-04-30 | All 7 historical bugs closed; WS5 empty |
| C5 — Async features fully lowered | ✅ DONE | f99261f04 + 8e195603b; multi-gen interleave residual = P2 follow-up |
| P1 — Geomean ≥1.5× CPython | ❌ OPEN | `enhancement-mamba-perf-bottoms-string-list-sort-int-mul` |
| P2 — No bench < 0.9× CPython | ❌ OPEN | Same as P1 |
| P3 — Compute-bound ≥2× CPython | 🟡 PARTIAL | int_sum_loop 3.17×, range_sum_loop 2.55×; bottoms drag geomean |
| P4 — `baseline.json` + CI gate | ✅ DONE | 8332b6f90 |

---

## Active workstream — sequenced
<!-- type: overview lang: markdown -->

### WS-A. C3 real-world programs (P1 priority)

The single largest gap. Sequenced inside-out:

1. **`enhancement-mamba-introspection-builtins-globals-locals-vars-dir` (P1)
   ✅ DONE 2026-04-30.**
   Branch `issue-enhancement-introspection-builtins-globals-locals-vars-dir`
   carries the implementation (3 commits: `a6bc5e14e` + `374fdeccb` +
   `aaad4293c`). All four builtins are content-faithful: `globals()` reads
   the runtime `MODULE_SYM_INFO` registry; function-scope `locals()` /
   `vars()` (no-arg) inline a per-frame dict snapshot at codegen time;
   `vars(obj)` raises `TypeError` for non-Instances; `dir(obj)` walks
   `type(obj).__mro__` and covers builtin types. 8/8 introspection
   conformance fixtures green; 48/48 builtins suite green. WS-A.2 is
   no longer blocked.

2. **`enhancement-mamba-c3-pytest-runs-unmodified` (P1, multi-week)**
   pytest hello-world + assert-rewrite + fixtures + parametrize.
   Unblocks the testing ecosystem and provides the harness for WS-A.3 / WS-A.4.

3. **`enhancement-mamba-c3-flask-runs-unmodified` (P2, multi-week)**
   Flask + werkzeug + WSGI dev server. Pulls priority items
   (`http.server`, `wsgiref`, `email`, `selectors`) from the stdlib
   long-tail tracker.

4. **`enhancement-mamba-c3-requests-runs-unmodified` (P2, multi-month)**
   The `ssl` module is the gating scope — TLS / SNI / CA bundle.
   Uses Flask as the local mock target.

### WS-B. Stdlib long-tail (P2, parallel to WS-A)

`enhancement-mamba-stdlib-stub-completion-tracker` carries a priority
queue ordered by C3 dependency: `os` / `sys` / `traceback` / `pathlib`
(P1 because they unblock pytest), then `http.server` / `wsgiref` /
`email` / `ssl` / `http.client` / `urllib.parse` (P2 for Flask +
requests). The mechanical pattern is established (16 modules per PR
in the 9e5840ba0 batch), so this work runs in parallel to WS-A and
gets pulled into urgency by C3 sub-issues.

### WS-C. Performance gap closure (P1 numerical gate)

`enhancement-mamba-perf-bottoms-string-list-sort-int-mul` —
investigate string_concat / list_sort_builtin / int_mul_loop /
factorial / generator_sum, fix or refute the benchmark methodology,
land regression-locked baseline updates.

### WS-D. Async-gen real event loop (P2 follow-up)

`enhancement-mamba-async-gen-real-event-loop-interleaving` —
distinct `MbAsyncGenerator` entity, awaitable `__anext__`. Estimated
2–3 days. Defer until a concrete consumer surfaces; current
single-gen routing covers the typical case.

---

## Spec / process hygiene (lower priority)
<!-- type: overview lang: markdown -->

These don't move the py3.12 needle but reduce future drift:

- **`refactor-mamba-hir-lower-pair-spec-refresh-post-desugar-misframing`**
  HIR/lower pair specs claim post-desugar IR; code retains
  comprehensions + with-statement + for-else. Rewrite to syntactic
  framing.
- **`refactor-mamba-retrospective-specs-audit-and-rewrite`**
  4 specs describe shipped work as open. Reclassify or close. Partial
  progress: ZeroDivisionError already fixed (`mb_floordiv` raises);
  remaining R-groups need status check.

---

## Package manager (separate epic, reference only)
<!-- type: overview lang: markdown -->

`epic-tracking-mamba-package-manager-uv-like` (closed) and the 4 open
phase-1 trackers (`pkg-mgr-phase-1-{2,3,4,5}-tracking`) are sequenced
independently of the py3.12 roadmap. Phase 1.1 shipped 2026-04-17;
Phase 1.2 (resolver) is greenfield. The C3 roadmap above intentionally
does **not** depend on pkgmgr — vendored test fixtures unblock pytest
without `pip install`.

---

## Future epics (gated)
<!-- type: overview lang: markdown -->

- `epic-py3-13-conformance` — gated on this epic's C1–C5 closing.
- `epic-py3-14-conformance` — doubly gated (3.13 → 3.14).

Neither has any code or dispatched work. They become real once py3.12
roadmap criteria above are satisfied.

---

## Out of scope (separate research)
<!-- type: overview lang: markdown -->

- `gevent` / `greenlet` — C-extension monkey-patching threading model
- `grpc` — C-extension protobuf-fast-path runtime
- `protobuf` C generator — same
- LSP server (#839), REPL (#838), WASM playground (#842) — developer tooling
- Error diagnostics quality (#840), incremental compilation (#837) — compiler UX

---

## Closure criterion for this roadmap
<!-- type: overview lang: markdown -->

When **WS-A 1–4 + WS-B priority queue + WS-C** all close, every box on
the master epic Success Criteria table flips to ✅, and the next session
opens with "py3.12 fully replaced — initiate py3.13 epic."

Estimated calendar: **multi-month**. Tracked progress lands as commits
on `main` and as closed sub-issues; no ETA published since the long
poles (ssl real implementation, perf-bottom investigation) have
multi-week uncertainty bands each.
