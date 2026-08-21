# mamba tech-design — the knowledge system

**Ownership**: this tree and `../external-contracts/` are authored and guarded
by the orchestrating agent. Implementation agents (mamba-dev) implement FROM
these documents and never edit them — report knowledge deltas (new hazards,
invariants, contract changes) in the issue's evidence comment; the guardian
folds them in and keeps the documents true after every landing.

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

A complete Python 3.12 runtime is large; this map names the WHOLE subsystem
taxonomy so gaps are visible. `docs?` marks current coverage — `map` =
ARCHITECTURE.md exists, `+topics` = topic docs too, `todo` = context named
but unwritten (author its ARCHITECTURE.md on first substantive work there).

### Frontend — source → typed HIR

| Domain | Owns | Key source | docs? |
|---|---|---|---|
| `frontend/` | lexer, parser, AST, HIR construction | `src/lexer/`, `src/parser/`, `src/hir/`, `src/source/` | map |
| `name-resolution/` | resolver passes, symbol tables, scoping (the resolver half of the two-pass system) | `src/resolve/` | map |
| `type-system/` | checker, signatures, walls, ingress enforcement | `src/types/` | +topics |

### Backend — HIR → machine

| Domain | Owns | Key source | docs? |
|---|---|---|---|
| `codegen/` | ast→hir→mir lowering, cranelift JIT, tracing emission | `src/lower/`, `src/mir/`, `src/codegen/` | +topics |
| `memory/` | NaN-boxing, refcount contracts, GC, escape analysis | `src/runtime/rc*`, `src/runtime/gc.rs`, `src/mir/escape_analysis.rs` | +topics |

### Runtime — object model & data types

| Domain | Owns | Key source | docs? |
|---|---|---|---|
| `object-model/` | class registry, identity/keys, MRO, slots, super, attribute dispatch | `src/runtime/class/`, `dict_ops` | +topics |
| `calling-convention/` | runtime arg binding — args/kwargs/defaults/kw-only/unpacking, frame adaptation | `runtime/builtins/` (mb_arg_bind, validate_and_adapt_declared_frame) | map |
| `numbers/` | the numeric tower — int/bigint, float, complex, bool, coercion; Decimal/Fraction | `runtime/bigint_ops.rs`, `integer_handle_registry.rs`, number mods | map |
| `strings/` | str/bytes/bytearray, unicode, codecs/encodings, formatting | `runtime/string_ops.rs`, `bytes_ops.rs`, codec mods | map |
| `collections/` | list/dict/set/tuple internals, views, hashing | `runtime/{list,dict,set,tuple}_ops.rs` | map |
| `iterators/` | iterator protocol, generators, coroutines, async iteration, state-machine lowering | `runtime/iter.rs`, `generator.rs` | map |
| `exceptions/` | construction, fields, propagation, rendering, traceback | `src/runtime/exception.rs` | +topics |
| `closures/` | capture cells, scoping (checker half), capture introspection | `src/runtime/closure.rs` | +topics |

### Runtime — services

| Domain | Owns | Key source | docs? |
|---|---|---|---|
| `concurrency/` | no-GIL threading, asyncio event loop, multiprocessing | `runtime/async_rt.rs`, `async_task.rs`, threading mods | map |
| `import-system/` | finders/loaders, module cache, circular imports, vendored resolution | import machinery, `vendor_lib.rs` | map |
| `stdlib/` | the 206 `*_mod.rs` surfaces, vendored modules, sentinel shims | `src/runtime/stdlib/` | +topics |
| `ffi/` | C3 native kit binding (mambalibs) | `src/ffi/` | todo |
| `pkgmanage/` | C4 package manager | `src/pkgmanage/` | map |

The EC machinery itself (runner verdict semantics, oracle cache, sweep
tooling, perf pins) is documented in `../external-contracts/HARNESS.md`.

Cross-domain knowledge lives in its dominant domain's topic doc and is
cross-referenced by path, never restated. When a `todo` context gets its
first real work, author its ARCHITECTURE.md first (as-is), then the topic doc.
