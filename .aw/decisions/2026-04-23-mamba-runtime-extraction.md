# 2026-04-23 — Mamba runtime extraction for jet-react-wasm

## Status

**Decided** — Yes, deferred to ~6 weeks out (once mamba hits 100%
CPython 3.12 conformance; currently 546 tests, 3 xfails). During
the interim, jet **vendors mamba source** and **prototypes against
it**. Upstream extraction-friendly refactors go to mamba as
priority PRs during that window.

## Context

The `epic-jet-wasm-canvas-renderer-react-compat` epic commits to a
full TSX → WASM pipeline. A large fraction of the runtime machinery
React needs (GC, closures, async executor, exception model, NaN-boxed
values) already exists in `crates/mamba` at production quality — 103K
LOC of Rust with a CPython-compat test suite. The question was
whether that work gets extracted into a shared crate
(`cclab-lang-runtime`) consumed by both mamba and `jet-react-wasm`,
or whether jet reimplements standalone.

Question document: `/tmp/mamba-runtime-extraction-questions.md`
(12 sections, 40-odd questions).

Reply document: `/tmp/mamba-runtime-extraction-reply.md` — excerpted
below where load-bearing.

## Decision

**Joint extraction into a new `cclab-lang-runtime` crate**, starting
after mamba's CPython 3.12 conformance milestone. Until then:

1. Jet **vendors** `crates/mamba/src/runtime/` source into a new
   location inside `crates/jet-react-wasm/` (or a sibling
   prototype crate) and builds against that copy.
2. Any extraction-friendly refactor the prototype work discovers
   (e.g. `GcTraced` trait to decouple `gc.rs` from `ObjData`) is
   upstreamed to mamba as a priority-review PR, even during their
   conformance push.
3. Jet nominates a **shepherding contact** before extraction starts;
   mamba will nominate one at the same time.
4. Jet commits to **not blocking mamba releases** — if a shared-crate
   change breaks mamba conformance, mamba rolls back and we redesign.

## Architectural confirmations from mamba

| Area | Finding | Source |
|---|---|---|
| Module scope | `rc.rs` + `gc.rs` extract **together** (share `ObjData` enum). `tokio_exec.rs` stays in mamba behind a feature flag. `output.rs` and `symbols.rs` don't extract — duplicate / stay. | Reply §1 |
| Value split | `RuntimeValue` (shared, tags 000/001/010/100) / `MbValue` (mamba adds None + NotImplemented) / `JetValue` (jet adds undefined) works. ~20 raw-bit sites need migration. | Reply §2 |
| GC root set | Explicit via `gc_add_root` / `gc_remove_root`, NOT stack-scan. **WASM-friendly drop-in.** | Reply §3.1 |
| GC finalizers | Not called from GC; finalization is pure refcount drop in `rc.rs`. No `finalize()` hook needed in the shared crate. | Reply §3.4 |
| Closure callable from Rust | Yes — `mb_call0` / `mb_call_with_args` direct. Positional only, no Python unwrap on hot path. | Reply §4.4 |
| Async tokio coupling | `async_rt.rs` is tokio-free; `tokio_exec.rs` is the only consumer. WASM target swaps in `wasm-bindgen-futures` without touching core. | Reply §5.1 |
| Exception shape | `MbException` is **already generic** — `{exc_type: String, message, cause, context, suppress_context, traceback: Vec<(file, line, func)>}`. Python class dispatch layered on top via a registry that stays in mamba. | Reply §6.1 |
| Unwinding | Rust `Result<T, MbException>` + thread-local `CURRENT_EXCEPTION`. **No panic, no setjmp, no WASM exceptions proposal needed.** Portable today. | Reply §6.3 |
| WASM backend | Mamba is NOT shipping a WASM backend in the next 6 months. Jet owns it. Recommended path: emit Rust source → `rustc --target wasm32-unknown-unknown` on the shared runtime + generated code. | Reply §7 |
| Rust source emission | **Confirmed as the right approach** — mamba's JIT pipeline (HIR→MIR→Cranelift) is overkill for TSX→WASM where jet controls the source AST. Different product, different pipeline, shared runtime. | Reply §7.4 |

## High-leverage callouts from the reply

> **The 🟡 call on Q2 (tag split) is the single most-important
> architectural decision; make sure the jet prototype lives with it
> for 2–3 weeks before we commit, because undoing it after downstream
> code hardens is expensive.**

→ Action: the prototype MUST use the `RuntimeValue` / `JetValue`
split, not punt on it and use `MbValue` directly.

## Extraction order (agreed)

1. `output.rs` — trivial, proves the crate structure.
2. `rc.rs` + `gc.rs` + the `ObjData` → `trait ContainerObject`
   split — **the hard one**; do this before anything else depends on
   its shape.
3. `value.rs` — `RuntimeValue` split; mamba's `MbValue` shrinks to
   a thin wrapper.
4. `closure.rs`.
5. `async_rt.rs` (leaving `tokio_exec.rs` behind in mamba).
6. `exception.rs` — struct moves, registry stays.

Order matters: the crate-structure PR lands first (output.rs) so
subsequent moves have a target; `rc.rs` + `gc.rs` are the hard one
because everything downstream depends on their shape.

## High-risk tests (from mamba)

When the extraction PRs land, run these first in isolation before
the full mamba conformance suite:

- `tests/fixtures/cpython/generators/` — all 20 files. Coroutine
  state machine is the most subtle; any `async_rt.rs` refactor needs
  these green.
- `exceptions/except_star_*` + `exception_group.py` — newest, most
  fragile exception paths.
- `scope_modifiers/nonlocal_types_broad.py` — regression guard for
  the recent box-cell fix (a914e87e). Touches closure capture
  semantics.

Mamba flags that the full conformance runner has a pre-existing
SIGABRT cascade they're tracking separately, so run the high-risk
subset in isolation.

## Immediate jet-side actions (in-flight while mamba finishes 3.12)

- [x] **jet-react-wasm-runtime skeleton** — hooks + fiber + flush loop
      shipped (commit `f2f0e549`). Pins the API the transpiler targets.
- [x] **jet-tsx-to-rust transpiler spike** — Counter.tsx round-trips
      to Rust; 8/8 tests pass (this commit).
- [ ] **Vendor mamba's runtime** into a prototype location
      (`crates/jet-react-wasm/vendored/mamba_runtime/`?) so the
      transpiler can target the real runtime APIs, with the Q2 tag
      split applied from day one.
- [ ] Commit `(Q2) RuntimeValue / JetValue split` in the prototype
      and **live with it for ≥ 2 weeks** before nominating a
      shepherd.
- [ ] Inventory the ~20 raw-bit-compare sites the reply flagged
      (§2.2) so migration cost is known.
- [ ] Upstream `trait ContainerObject` as an extraction-friendly
      refactor PR to mamba once the prototype proves the shape.

## Reply decision

> **Yes — but start in ~6 weeks** (once mamba hits 100% CPython 3.12
> conformance, currently 546 tests / 3 xfails). Until then:
>
> - jet can **vendor mamba's source** into `jet-react-wasm` and
>   prototype. Use it as the architectural reference.
> - Upstream any **extraction-friendly refactors** (e.g., `GcTraced`
>   trait) to mamba — we'll treat those as priority reviews even
>   during our conformance push.
> - Ping us with **one shepherding contact** from jet's side once
>   you're ready. mamba shepherd will be nominated when we start.

## Out of scope for this decision

- Mamba's own WASM backend roadmap — deferred beyond 6 months.
- Whether future consumers (hypothetical TS runtime, Lua embed) join
  `cclab-lang-runtime` — left for when a second consumer actually
  materialises.
- Specific feature-flag layout on the shared crate — monolithic
  initially; split if a consumer needs a `no-async` build.

## Artifacts

- Question doc: `/tmp/mamba-runtime-extraction-questions.md`
- Reply: `/tmp/mamba-runtime-extraction-reply.md`
- This decision: `.aw/decisions/2026-04-23-mamba-runtime-extraction.md`
- First jet-side implementations: `crates/jet-react-wasm-runtime/`,
  `crates/jet-tsx-to-rust/`.
