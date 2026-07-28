# Class construction — registration order, dispatch, and the super contract

How a class definition becomes a live registry entry, how instances are
built, and the rules the super/`__new__`/`__init__` machinery must uphold.

## Registration pipeline (lowering → runtime)

`PendingClassRegistration` drains in a fixed order at each class's textual
`ClassDefPlaceholder` (and a fallback loop for hand-built HIR):

1. `mb_class_define_multi` — create the record, then immediately propagate any
   pending exception raised while computing its static-base MRO.
2. `mb_class_update_bases` — resolve runtime-valued bases, finalize MRO, then
   immediately propagate any pending exception raised by that recomputation.
3. `emit_class_slots_for` — register `__slots__` (MUST follow step 2: the
   slot merge reads the MRO; running it before update_bases drops inherited
   slots). Statically-based classes keep immediate registration (R3).
4. class attrs, metaclass, PEP 487 `__init_subclass__`/`__set_name__` hooks
   (gated by `creation_hooks_pending`), then decorators.

Invariant: a class with runtime bases follows define → update_bases →
register_slots; the queue-and-drain pattern (`pending_class_*` vecs +
`class_runtime_key_value` cached vreg) exists to enforce it.

### MRO rejection barrier

Class registration is a transaction boundary even though the registry may
hold a provisional record internally. `compute_mro` owns C3 validation and
sets a catchable `TypeError` for duplicate bases or an inconsistent
linearization. Lowering MUST place an exception-propagation barrier directly
after both runtime calls that can compute an MRO:
`mb_class_define_multi[_named]` and `mb_class_update_bases`.

If either barrier observes an exception, execution leaves the class statement
before slots, body side effects, attributes, finalizers, decorators, or the
source name binding run. An enclosing `try/except TypeError` can catch the
error; an uncaught top-level error remains observable to the execution
boundary before runtime cleanup. The fallback MRO stored by the runtime is
recovery-only internal state and MUST NOT make the rejected class observable.

Valid single inheritance, diamonds, and consistent multiple inheritance cross
both barriers unchanged. Verification therefore needs paired witnesses:
an inconsistent `A(X, Y)` / `B(Y, X)` / `C(A, B)` hierarchy that is caught at
the `C` statement, and a valid diamond whose MRO and class body remain live.

Step 4's `__init_subclass__` dispatch (`dispatch_type_new_creation_hooks`,
mod.rs:1631) carries the same closure-handle hazard as `__init__` (Instance
construction, below): the hook's dispatch address MUST come from
`extract_registered_func_addr`, never `extract_func_addr`. A hook body
referencing bare `__class__` or zero-arg `super()` compiles as a closure;
the raw extractor's int fallback is never in `CALLABLE_REGISTRY`, so
`is_registered` reads false and the whole hook silently no-ops — no error,
no dispatch.

Known gap: `cls.__slots__` reports the merged effective layout instead of the
declared tuple (layout itself is correct) — tracked: #1523.

## Instance construction

`call_init_for_instance` caches `__init__`'s dispatch address. The address
MUST be computed via `extract_registered_func_addr` (which unwraps closure
handles), never `extract_func_addr`: since class-cell capture became
unconditional, any `__init__` using bare `__class__` or zero-arg `super()` is
a closure handle, and the raw extractor returns a garbage address absent from
`CALLABLE_REGISTRY` → silent full-`__init__` skip (no body, no MRO init, no
error). This was the corpus-dominating regression class.

Known ceiling: metaclass `__call__` construction is arity-limited to 3 ctor
args (mod.rs:3941, falls to none above) — suspected live gap, unverified.

## Super machinery + error semantics

`super_builtin_native_method` routes `__new__`-reaching-type/object through
`make_unbound_method` → `type_new_unbound`/`object_new_unbound`.

**Red line:** `__init__` is deliberately EXCLUDED from that arm. Including it
regresses the ubiquitous no-op `super().__init__()` idiom — the
`SUPER_MISSING_INIT_METHOD` fallback must keep binding the implicit receiver.
Do not "complete the symmetry."

CPython error parity is explicit machinery: zero-arg super with no enclosing
class → `mb_super_no_args_error`; >2 args → `mb_super_argcount_error`; bad
types → `mb_super_checked`; duplicate bases, `object.__new__` extra args,
non-None `__init__` return each raise their specific TypeError. All messages
route through `class_display_name` (never raw registry keys — see
identity-and-keys.md).

Known gaps: `super().__call__` metaclass dispatch and direct `type.__init__`
unbound resolution (tracked: #1525); `super().__class__` returns the proxy's
internal class_name instead of the real `super` type — the `__class__`
intercept in mb_getattr must yield to `mb_super_getattr` for super proxies
(tracked: #1581).

## Builtin construction dispatch

Unbound access `BuiltinType.member` and bound access `instance.member` must
resolve to the same callable (modulo binding). A member table that returns
None for unbound/direct access where bound access resolves produces the
`'NoneType' object is not callable` family: `type(None)()` singletons,
`dict.__or__(a,b)` explicit dunder calls, multi-class composite construction.
Diagnose these together — likely one resolution gap (tracked: #1550, #1582;
exception-subclass variant in exceptions/construction-and-rendering.md
§Known gaps).

Per-builtin kwargs facts are real API contracts: `dict(**kw)` constructs;
`list/set/frozenset` reject kwargs. Never share the accept/reject arm
(dict kwargs construction tracked: #1549).

## EC surface

`_regression/core/{class_system,mro_super,language}`, `behavior/core/descr`;
super-idiom and construction probes vs the oracle.
