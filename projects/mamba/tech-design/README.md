# mamba tech-design — domain-driven layout

Condensed implementation designs authored by the orchestrating agent;
implementation agents (mamba-dev) implement from them. Each TD: **Mechanism**
(why it breaks, 2-5 lines) / **Invariant** (the rule the fix establishes) /
**Fix pattern** (file:symbol level) / **Red lines** (what NOT to "complete") /
**Verification contract** (exact fixtures + sweeps). One page max — density
over coverage. Filename: `<issue-or-family>-<slug>.md`.

## Context map (bounded contexts)

| Domain | Owns | Key source | EC surface (fixture dimensions) |
|---|---|---|---|
| `type-system/` | strict-type checker, signatures, walls, ingress enforcement | `src/types/` | `type/` dimension IS this domain's contract (walls must reject); over-walling = this domain rejecting other domains' legal fixtures |
| `object-model/` | class registry, MRO, slots, super, metaclass, runtime keys, dict/value model | `src/runtime/class/`, `dict_ops` | `_regression/core/{class_system,mro_super,language}`, `behavior/core/descr` |
| `memory/` | refcount contracts, GC tracking, escape analysis | `src/mir/escape_analysis.rs`, `src/runtime/rc` | `gc/`, `stability/`, hang/SIGTRAP symptoms anywhere |
| `exceptions/` | construction, fields, propagation, traceback | `src/runtime/exception.rs` | `_regression/core/exception*`, `behavior/core/exceptions` |
| `closures/` | capture cells, scoping, walrus, introspection of captures | `src/runtime/closure.rs`, lowering scope passes | `pep/572`, `behavior/std-libs/inspect` (capture-related) |
| `pkgmanage/` | C4 package manager | `src/pkgmanage/` | pkgmanage test suite |

Unassigned future contexts (create on first TD): `codegen/` (cranelift/JIT),
`stdlib/` (per-module runtime), `concurrency/` (threading/asyncio/GIL).

Cross-domain families keep ONE doc in their dominant domain and cross-reference
(e.g. `object-model/runtime-key-aliasing-family.md` is cited by stdlib fixes).
