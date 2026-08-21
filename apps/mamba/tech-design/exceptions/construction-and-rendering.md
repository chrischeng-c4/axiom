# Exception construction and rendering — fields, chaining, repr

How exception instances acquire their structured state and how they render.
The through-line: an exception must carry its fields at construction, because
rendering recomputes from those fields.

## Fields must be present at construction

`str(Unicode*Error)` always recomputes from encoding/object/start/end/reason —
a message-only raise renders EMPTY. Invariant: raise Unicode errors through
the structured helper (`raise_unicode_encode_error_instance`) with explicit
CPython reason strings (`ordinal not in range(128)` / `(256)`), never bare
`mb_raise(type, msg)`.

Keyword-only attrs (`AttributeError(name=,obj=)`, `ImportError(name=,path=)`,
`NameError(name=)`) must round-trip: construct with kwarg → read attr back.
The historical failure was a raw native-hash probe of the kwargs dict — a
DictKey hash-domain miss (see object-model/identity-and-keys.md); the fix is
`dict_get_exact_str`. Round-trip probing is the mandatory verification shape
for any new kwargs-bearing exception.

`BaseException.__new__` pre-stores `args` at allocation; a user `__init__`
that never chains still yields `str(inst) == first_arg`. Absence of this
pre-store makes such instances render as `<C instance>`.

## Instance method synthesis

`exc.__init__()` on an instance requires `__init__` in `mb_getattr`'s
bound-native synthesis arm (alongside `__setstate__|add_note`), otherwise the
working BaseException-`__init__` handler is unreachable from getattr+call.

## Known gaps (unbound chain + composite)

Subclass `__init__` shapes still diverge (tracked: #1557):
- `Exception.__init__(self, a)` UNBOUND chain then `self.tag=1` loses `tag`
  (`super().__init__(a)` works — defect specific to the unbound class-attr
  call form; diagnose whether the unbound call rebinds the receiver or aborts
  the caller body via `--emit mir` P1-vs-P2).
- No-chain subclass renders `<C instance>` (missing `__new__` args pre-store).
- Composite multi-class + inline construction crashes with
  `'NoneType' object is not callable` — same family as the builtin
  unbound-dispatch gap in object-model/class-construction.md; diagnose
  together.

## Rendering

ExceptionGroup repr renders each contained exception via its OWN repr
(`ClassName(arg_repr,…)`), recursively for nested groups; str(group) =
`message (N sub-exception(s))`. Rendering contained args directly (bare `2`
for `KeyboardInterrupt(2)`) is a defect (tracked: #1522).

## State machine note

Propagation is poll-based (`mb_has_exception` checks emitted by lowering, no
unwinding); raises are dual-channel (name-string summary + retained instance).
See ARCHITECTURE.md for the six-slot state machine; the settrace exception
event fired during unwind is codegen's contract
(codegen/tracing-and-frames.md).

## EC surface

`_regression/core/exception*`, `behavior/core/exceptions`.
