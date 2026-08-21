# calling-convention — architecture (as-is, 2026-07-15)

Runtime arg binding: turning a call site's positionals/keywords/splats into the
callee's declared entry frame. Two tiers — a **compile-time** static reorder for
known idents with literal keyword names (`lower/ast_to_hir.rs`), and a
**runtime** binder for dynamic-key `**mapping`, unknown callees, and every
`(*args,**kw)` spread (`runtime/builtins/mod.rs`). Compile-time *type* rejection
of wrong scalar args is the type-system's job — see
`../type-system/walls-and-widening.md`; this domain owns the *runtime* binding
algorithm and the scalar wall re-checked at ingress.

## Responsibilities

- Bind positional / positional-or-keyword / `*args` / keyword-only / `**kwargs`
  to the declared entry frame at runtime (`bind_declared_call_frame`, mod.rs:6009).
- Fill defaults (right-aligned), and synthesize CPython-exact bind TypeErrors
  (missing / duplicate / unexpected / too-many).
- Re-enforce the declared scalar contract and adapt each slot to the JIT entry
  ABI (`validate_and_adapt_declared_frame`, mod.rs:6270).
- Per-builtin keyword acceptance/rejection routing at lower time (str/print/open/
  dict… accept; list/set/frozenset reject).
- Thread the 7-tuple param metadata from lowering into the runtime registry.

## Key structures & invariants

| Structure | Rule that must hold |
|---|---|
| `closure.rs:838 MbParamInfo{name,kind,has_default,default,annotation,entry_abi,contract}` | The per-call-frame contract. `kind` = CPython `inspect.Parameter` ordinal: 0 POS_ONLY, 1 POS_OR_KW, 2 VAR_POSITIONAL, 3 KW_ONLY, 4 VAR_KEYWORD. `contract` (semantic scalar) drives runtime rejection; `entry_abi` (representation) drives unbox. Both are independent — never conflate. |
| 7-tuple `(name,kind,has_default,default,annotation,entry_abi,contract)` (`hir_to_mir.rs:2058`, lambda `:11786`) | Emitted per param → `mb_func_set_params` (`closure.rs:861`). `contract` set only from `p.declared_ty` scalar AND `annotation.is_some()` (`hir_to_mir.rs:2023`); `entry_abi` from `p.entry_ty`+`boxed_primitive_entry` (`:2040`). Lambdas emit all-`boxed`, no contract. 5/6-field tuples accepted for old modules; default is retained, prev released on overwrite. |
| `mod.rs:6133 RuntimeScalarContract` = Int/Bool/Float/Str/Bytes/None | The only 6 runtime-enforced contracts; `runtime_scalar_contract` (`:6143`) maps the string. Anything else (Any/container/forward-ref) stays dynamic. |
| `ast_to_hir.rs:3564 arg_bind_sigs: name→[(name,kwonly,has_default,star,dstar,posonly)]` | Top-level bare-Ident defs only; source-of-truth for the compile-time violation check + kw-only split. Flow-sensitive; removed on rebind (`:4176`). |
| Error sentinel: `bind_declared_call_frame` returns `Some(vec![])` on a raised bind error, `None` only when the callee has no param metadata, `Some(frame)` on success (`mod.rs:6043,6102,6130`) | Callers MUST check `current_exception_type()` after binding (`mod.rs:6491`); an empty frame is "raised — stop", not "0 args". |

## Control flow

1. **Lower** (`ast_to_hir.rs` call arm ~7653+): known-ident + literal kwargs →
   per-builtin special case (str→`mb_str_construct`, print→`mb_print_kwargs_file`,
   open→`mb_open_kwargs`, dict→`Dict`/`mb_dict_from_pairs`+`mb_dict_merge`) or
   reject arm (list/set/frozenset kwargs → `mb_arg_bind_error`, `:7686`).
2. Known user def, literal keywords, no `**` → static reorder into positional
   slots (kw-only split via `arg_bind_sigs`, `:9149`); `arg_bind_violation`
   (`:746`) emits the compile-time TypeError string for the 5 CPython cases.
3. Dynamic keys (`**mapping`), unknown callee, or declared variadic →
   `build_spread_kwargs_call` (`:10884`) / `build_kwargs_dict` (`:10958`): merge
   keywords + each `**` in source order via `mb_dict_merge` → `mb_call_spread_kwargs`.
4. **Runtime** `mb_call_spread_kwargs` (`mod.rs:6519`): unwrap `functools.partial`
   (prepend args, merge keywords, recurse); no keywords → `mb_call_spread`;
   `__exec_function__`/ast-node/native-ctor special cases; else `invoke_args_kwargs`.
5. `invoke_args_kwargs` (`:6463`): native → append kwargs dict as trailing arg;
   else `detect_star_kw` (`:6440`), `bind_declared_call_frame` → check exception →
   `validate_and_adapt_declared_frame` → `with_closure_cells` + `dispatch_jit_frame`.
6. `bind_declared_call_frame` (`:6009`): positional fill L→R, then keyword-by-name,
   then defaults; overflow → `*args` list; unused keywords → `**kwargs` dict.
7. `dispatch_jit_frame` (`:6354`): arity-keyed `transmute` for 0..=8 slots; reboxes
   raw-int return unless boxed-return callee; **>8 params silently returns None**.

## CPython-parity semantics

- **Bind order**: positionals bind first (POS_ONLY + POS_OR_KW), then keywords by
  name, then defaults; contract mirrors CPython's slot algorithm.
- **Error precedence** (as coded): multiple-values (at the slot) → too-many-positional
  (post-loop, `:6096`) → missing-positional (`:6104`) → missing-kwonly (`:6109`) →
  unexpected-keyword (`:6122`). Oxford comma for N≥3 missing (`missing_positional_args_message`, `:5991`).
- **Defaults right-aligned** to POS_ONLY+POS_OR_KW; live `__defaults__` rewrite
  re-aligns the same way (`mb_func_set_pos_defaults`, `closure.rs:926`).
- **Positional-only-as-keyword** lands in `**kwargs` (never the slot); a bind error
  only without `**kwargs` (`arg_bind_violation` case 2, `:784`).
- **kwargs order**: `mb_dict_merge` / `merge_kwargs_dicts` preserve insertion order,
  **call-time (`b`) wins** on collision (`:5961`).
- **Scalar wall widenings** (mirror `SupportsIndex`/`SupportsFloat`): int accepts
  bool/bigint/IntEnum/int-subclass (`strict_int_payload`, `:6155`); float accepts
  int/bool/bigint (`:6195`); bytes/str accept their subclasses (`:6221`).
- **Per-builtin kwargs**: dict ACCEPTS, list/set/frozenset REJECT (#1549) — never
  share the arms; see `../type-system/walls-and-widening.md`.

## Known hazards

- **Arity cap at 8** (`dispatch_jit_frame`, `:6424`): a 9+ param declared frame
  binds correctly then dispatches to `None` — silent wrong result, clean compile.
- **Empty-frame sentinel** (`:6043`): treating `Some(vec![])` as "0-arg call"
  instead of "raised" double-dispatches a callee whose bind already errored.
- **Non-str kwargs keys silently dropped** (`kwargs_dict_pairs`, `:5949` filters to
  `DictKey::Str`): `f(**{1:2})` loses the key; CPython raises `keywords must be strings`.
- **entry_abi/contract desync**: `contract` gates *rejection* (annotation-gated),
  `entry_abi` gates *unbox representation*; old metadata falls back to `boxed`.
  Emitting one without the other → wrong unboxing or a missed wall.
- **Static default-fill blind to runtime mutation**: `__defaults__ = (...)` installs
  defaults the source signature lacks; must route via `funcs_with_mutated_defaults`
  → `build_mutated_defaults_call` (`ast_to_hir.rs:9123`), else missing-arg misfire.
- **Forgotten `**`-splat guard**: any new per-builtin ident special-case that omits
  the `DoubleStarArg` guard drops dynamic keys (the reason `:9030` exists).
- **variadic scalar contract validates elements only** (`:6286` kind 2, `:6314` kind 4):
  the `*args`/`**kwargs` *container* is never scalar-adapted, only its members.

## Extension points

| Adding | Where it plugs in |
|---|---|
| New per-builtin keyword shape | ident arm in `ast_to_hir.rs` call-lowering (~`:7957`) building the dedicated `mb_*_kwargs` helper call; keep the `**`-splat guard. |
| New runtime scalar contract | `RuntimeScalarContract` variant + `runtime_scalar_contract` (`:6143`) + `strict_scalar_value` (`:6221`) + `adapt_value_for_entry_abi` (`:6247`) + the `contract` match in `hir_to_mir.rs:2027`. |
| New param kind semantics | `MbParamInfo.kind` handling + arm in `bind_declared_call_frame` (`:6023`) + kind ordinal at `hir_to_mir.rs:11759`. |
| New entry ABI | `adapt_value_for_entry_abi` arm (`:6247`) + the `entry_abi` match (`hir_to_mir.rs:2042`); arity/box registration is codegen's — see `../codegen/ARCHITECTURE.md`. |

## EC surface

- `behavior/core/args_kwargs_binding` (dynamic keyword bind-by-name), `behavior/core/funcattrs` (`__defaults__`/`__kwdefaults__` rewrite).
- `behavior/std-libs/{extcall, call, userfunctions, keywordonlyarg, positional_only_arg, getargs, functools}` — spread syntax, kw-only, pos-only, partial.
- `type/core/arg_annotation` — the NEGATIVE walls (per-kind scalar rejection):
  `func_int_arg_called_with_str`, `{keyword_only,positional_only,varargs,kwargs}_int_arg_called_with_str`,
  `func_str_arg_called_with_bytes`, `default_int_arg_uses_str_default` — weakening one is a contract breach; cross-ref `../type-system/walls-and-widening.md`.
- Full gate: `cargo test -p mamba --release --test conformance` (~3 min, oracle = python3.12 byte-diff).
