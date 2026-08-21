# exceptions — architecture (as-is, 2026-07-15)

Scope: `src/runtime/exception.rs` (3.7k L), `src/runtime/stdlib/traceback_mod.rs` (4.1k L),
raise/catch slots in `src/runtime/class/mod.rs` (~4788–5223), lowering in `src/lower/hir_to_mir.rs`.
Fix TDs in this dir: `construction-and-rendering.md` (main body landed; §Known gaps OPEN).

## Responsibilities

- Exception construction: typed per-class fields, `args` tuple, kwargs consumption (`exception.rs:mb_exception_new_with_args{,_and_kwargs}`, `populate_exception_fields`).
- Raise/pending/catch state machine over thread-local slots + chaining (`__cause__`/`__context__`/`__suppress_context__`).
- Builtin exception hierarchy: init-time registry registration (`exception.rs:register`, R1) plus hard-coded `is_subclass_of` fallback table (`exception.rs:1253`).
- Traceback capture (frame stack), `__traceback__` stamping, and sys.settrace `'exception'` event emission (`traceback_mod.rs`).
- PEP 654 ExceptionGroup: construct/narrow/validate, `split`/`subgroup`, `except*` (`exception.rs:1518+`).
- Unicode*Error `str()` recomputation from structured fields (`exception.rs:unicode_error_str`).

## Key structures & invariants

`exception.rs:11 MbException` = `{exc_type: String, message: String, cause/context: Option<Box<MbException>>, suppress_context: bool, traceback: Vec<(file, line, func)>}`. **Type identity is a name string, not a class object** — everything downstream (matching, subclass checks, EG conditions) compares names.

Thread-local slots (the state machine):

| Slot | file:symbol | Holds | Written by | Cleared by |
|---|---|---|---|---|
| `CURRENT_EXCEPTION` | exception.rs:51 | pending `MbException` — THE pending flag | `mb_raise*`, `set_current_exception` | `mb_catch_exception`, `mb_clear_exception`, `mb_take_uncaught_traceback` |
| `LAST_RAISED_INSTANCE` | class/mod.rs:5221 | retained full instance (custom fields survive) | `mb_raise_instance*` | `mb_catch_exception_instance` (take), every `mb_raise*` via `clear_last_raised_instance` |
| `LAST_CAUGHT_VALUE` | class/mod.rs:5134 | borrowed bits — what `sys.exception()` reports | `mb_catch_exception_instance` | save/restore stack only |
| `LAST_HANDLED_EXCEPTION` | exception.rs:58 | clone for `format_exc`/`exc_info` | `mb_catch_exception` | never auto-cleared; save/restore reinstates |
| `HANDLED_EXC_SAVE_STACK` | exception.rs:65 | (handled-snapshot, caught-bits) per try-entry, parked retain | `mb_save_handled_exc` | `mb_restore_handled_exc(token)` |
| `TRACE_FRAME_STACK` | traceback_mod.rs:71 | `TraceFrame{file,line,name,locals,hook,exception_notified}` | push/pop per call | frame pop |

Invariants that must hold:

- **Dual-channel raise**: an instance raise sets BOTH `CURRENT_EXCEPTION` (name+msg summary, class/mod.rs:4832) and `LAST_RAISED_INSTANCE` (retained instance). Every `mb_raise*` first calls `clear_last_raised_instance` so a summary can never pair with a stale instance.
- **Poll-based propagation, no unwinding**: lowering emits `mb_has_exception()` checks after calls inside try scope; a raise = `mb_raise*` call + terminator (goto handler / with-exit / return None). Nothing longjmps.
- **unicode_error_str recompute rule**: `str(Unicode*Error)` is ALWAYS recomputed from `encoding/object/start/end/reason` instance fields (string_ops.rs:5122 → exception.rs:490); a message-only `mb_raise` leaves them unset → renders `""`. Structured raise helper is mandatory (see `construction-and-rendering.md`).
- **kwargs probes use `dict_get_exact_str`** (exception.rs:403) — raw `&str` hash misses `DictKey::Str` (hash-domain defect, cross-domain).
- **StopIteration dual signal**: any raise spelling of StopIteration must flip `iter::signal_stop_iteration()` (mb_raise:764, mb_raise_instance:4799/4838); catching one must `iter::check_and_clear_stop()` (class/mod.rs:5206).
- **Chain preservation**: converting instance → `MbException` walks the full `__cause__`/`__context__` chain (`mbvalue_to_mbexception`, depth cap 4096); `raise X from Y` always sets `suppress_context=true`.
- Save-stack entries park one retain on the caught value (exception.rs:81); restore releases exactly once per popped slot (exception.rs:122).

## Control flow

1. `raise E("m")`, builtin type → `hir_to_mir.rs:4601 HirStmt::Raise` picks `mb_raise` / `mb_raise_from` / `mb_raise_with_context` / `mb_raise_from_with_context` (context = lowering-time `active_except_vreg` presence) → new `MbException` into `CURRENT_EXCEPTION`.
2. `raise E(...)`, user class → `mb_instance_new_with_init` → `class/mod.rs:4788 mb_raise_instance` (Str payload = bare type name; `class_name=="type"` resolves via `__name__`; else instance) → summary + `LAST_RAISED_INSTANCE`.
3. Immediately after any value-carrying raise: `hir_to_mir.rs:4959` → `traceback_mod.rs:1179 mb_traceback_capture_raise`: snapshot `TRACE_FRAME_STACK` → `set_current_traceback(entries)` + stamp `__traceback__` on the peeked instance + fire `'exception'` trace event; marks origin frame `exception_notified=true`.
4. Terminator: goto innermost try handler; else innermost `with` exit block (so `__exit__` runs); else restore handler regions + return None (caller polls `mb_has_exception`).
5. Frame exit during unwind: `mb_traceback_pop_frame_with_return` (traceback_mod.rs:1113) → `mb_traceback_notify_unwind_exception` fires `'exception'` once per frame (gated on `exception_notified`) → `'return'` event → pop.
6. try-entry: `mb_save_handled_exc` (exception.rs:74) pushes snapshot, returns token vreg.
7. except dispatch: `mb_catch_exception_instance` (class/mod.rs:5193) — prefers `LAST_RAISED_INSTANCE`; fallback `mb_catch_exception` (exception.rs:939) synthesizes an Instance via `store_exception_as_value` + trims tb (`trim_traceback_to_current_handler`, traceback_mod.rs:1236) + records `LAST_HANDLED_EXCEPTION`. Match test: `mb_exception_matches` → `collect_matcher_targets` (rejects non-BaseException matchers with TypeError) → `is_subclass_of`.
8. No handler matched: `mb_reraise` (= `mb_raise_instance`, preserves fields). Bare `raise` in generator bodies: `mb_reraise_handled` (exception.rs:887).
9. Handler-region exit edges: `mb_restore_handled_exc(token)`; success path also `mb_clear_exception`. All three clear paths reset the frame's `exception_notified` (#1535).
10. Uncaught at module end: driver → `mb_take_uncaught_traceback` (exception.rs:919) — fixed `<module>` header, not the real captured entries.
11. `except*`: `mb_except_star` → `parse_eg_condition` (Type/Types/Predicate) → `eg_split_rec` (exception.rs:2267, recursive matched/rest via `eg_derive`); rest re-raised through `mb_reraise`. Constructor path: `mb_exception_group_construct` (arity/str/sequence/non-empty/member validation, PEP 654 narrowing via `narrow_eg_class_name`).

## Known hazards

- **Name-string type identity** — same-named classes are indistinguishable; `is_subclass_of(_, "Exception")` early-trues for any builtin name (see comment exception.rs:1576). WHY: EG member checks and matchers silently over-match; `eg_member_is_bare_base` works around it with an explicit BaseException-root list.
- **Dual-channel desync** — any new raise path that sets one slot but not the other. WHY: catch sees a stale/absent instance and reconstructs from summary, dropping custom fields.
- **Pending exception blocks all calls** — `mb_call_spread_impl`'s post-bind check aborts any call while an exception is pending; `suspend_current_exception`/`restore_suspended_exception` (exception.rs:1082, #1535) is the only carve-out (trace callbacks). WHY: runtime code that must call user code during unwind silently no-ops.
- **`exception_notified` staleness** — flag must be reset on every clear path (catch/clear/uncaught). WHY: a later distinct exception in the same still-active frame loses its `'exception'` event.
- **kwargs hash-domain probe** — raw dict `.get(&str)` misses; use `dict_get_exact_str`. Detail: `construction-and-rendering.md`.
- **Message-only Unicode*Error raise renders `str()` empty** — recompute rule above. Detail: `construction-and-rendering.md`.
- **`store_exception_as_value` round-trip loses custom fields** — `MbException` carries only type/message/chain/tb. WHY: fallback catch (no `LAST_RAISED_INSTANCE`) yields a synthetic instance; user attrs vanish.
- **`LAST_HANDLED_EXCEPTION` never auto-cleared** — deliberate for post-handler `format_exc()`, but `mb_reraise_handled` can resurrect it far from the original handler.
- **Save-stack retain parking** — abnormal region exits discard deeper slots; each parked retain must release exactly once. WHY: earlier miscount surfaced as intermittent double-free in nested try/except (class/mod.rs:5140 comment).
- **StopIteration flag leak** — user `except StopIteration:` without the class/mod.rs:5206 clear makes the next generator resume read phantom exhaustion.
- **Synthetic tracebacks** — traceback_mod.rs header carve-out: no real frame walk/linecache; several `traceback` functions return empty surfaces; `mb_take_uncaught_traceback` prints a fixed one-frame header. WHY: callers pretty-printing real tracebacks observe empty/wrong output while gates still pass.
- **Open #1557 shapes** — unbound `Exception.__init__(self,…)` chain loses attrs; `__new__` args not pre-stored (P3 `str()` falls to generic repr); composite NoneType-callable crash. See `construction-and-rendering.md` §Known gaps (do not "fix" past its red lines).

## Extension points

| To add | Plug in at |
|---|---|
| Builtin exception with structured fields | new arm in `populate_exception_fields` (exception.rs:294) + allowed-kwargs table (exception.rs:561) + `register()` hierarchy entry + `is_subclass_of` arm when the parent isn't Exception/BaseException |
| Structured raise from native code | copy `string_ops.rs:283 raise_unicode_encode_error_instance` pattern: build full-field instance (incl. `args`, `__type__`, chain fields) → `class::mb_raise_instance` — never bare `mb_raise(type, msg)` when the class has field-derived `str()` |
| Custom `str()` rendering per class | instance arm of `string_ops.rs:value_to_string` (~5100–5150; Unicode*Error and EG hooks live there) |
| New exception intrinsic callable from codegen | register extern in `runtime/symbols.rs`; emit `MirInst::CallExtern{name}` from `lower/hir_to_mir.rs` |
| New trace event around exceptions | `threading_mod::mb_threading_emit_*` + a `TraceFrame` gate flag (follow `exception_notified`) |
| EG behaviors (subgroup/split/derive variants) | `EgCondition` + `eg_split_rec`/`eg_derive` (exception.rs:2003+) |

## EC surface

Per `external-contracts/README.md` domain map (positive contract only, no wall dimension):

- `tests/cpython/_regression/core/exception*` → dirs: `exceptions`, `exception_chaining`, `exception_control_flow`, `exception_group`, `custom_exception`.
- `tests/cpython/behavior/core/exceptions` (attr/name/obj probes, per-shape tests).
- Traceback surface: `tests/cpython/behavior/std-libs/traceback` (#1441 3-gate contract).
- Oracle: python3.12 byte-diff via `cargo test -p mamba --release --test conformance`; per-fix evidence = before/after gate readings on the issue.
