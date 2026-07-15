# mamba tech-design — the knowledge system

This tree is the durable knowledge base of how mamba works: mechanisms,
invariants, hazards, and the verification surfaces that prove them. It is NOT
an issue log — implementation status lives in the tracker, never here.
Documents are written in the present tense about the system as it is; a known
violation of an invariant is stated as knowledge with a tracker footnote
(`tracked: #NNNN`), not as a todo.

## Layout

Each bounded context has:

- `ARCHITECTURE.md` — the map: responsibilities, key structures, control flow,
  hazards index, extension points, EC surface.
- A small number of **topic documents** — one coherent concern each, absorbing
  what would otherwise be per-fix fragments. When a change lands, its lasting
  knowledge (the invariant it established, the hazard it revealed, the pattern
  it proved) is folded into the owning topic document; the change history
  itself stays in git/issues.

Filename: `<topic-slug>.md` — semantic, never issue-ids. One page per topic;
density over coverage; file:symbol references; decision tables over prose.

## Context map (bounded contexts)

| Domain | Owns | Key source | EC surface |
|---|---|---|---|
| `type-system/` | checker, signatures, walls, ingress enforcement | `src/types/` | `type/` dimension = negative contract (walls must reject); other dimensions must never be compile-rejected |
| `object-model/` | class registry, identity/keys, MRO, slots, super, dispatch, dict/value model | `src/runtime/class/`, `dict_ops` | `_regression/core/{class_system,mro_super,language,descriptors}`, `behavior/core/descr` |
| `memory/` | NaN-boxing, refcount contracts, GC, escape analysis | `src/runtime/rc*`, `src/runtime/gc.rs`, `src/mir/escape_analysis.rs` | `behavior|surface|type/std-libs/gc`, `_regression/core/stability`; corpus-wide hang/SIGTRAP jurisdiction |
| `exceptions/` | construction, fields, propagation, rendering, traceback | `src/runtime/exception.rs` | `_regression/core/exception*`, `behavior/core/exceptions` |
| `closures/` | capture cells, scoping passes, capture introspection | `src/runtime/closure.rs`, resolver/checker scope arms | `pep/572`, capture-introspection fixtures |
| `codegen/` | ast→hir→mir lowering, cranelift JIT, tracing emission | `src/lower/`, `src/codegen/cranelift/` | `--emit ast|hir|mir` tooling; `behavior/core/sys_settrace` |
| `stdlib/` | the `*_mod.rs` runtime surface, vendored modules, kits | `src/runtime/stdlib/` | `behavior|errors|real_world/std-libs/<mod>` |
| `pkgmanage/` | C4 package manager | `src/pkgmanage/` | pkgmanage suite |

The EC machinery itself (runner verdict semantics, oracle cache, sweep
tooling, perf pins) is documented in `../external-contracts/HARNESS.md`.

Cross-domain knowledge lives in its dominant domain's topic doc and is
cross-referenced by path, never restated.
