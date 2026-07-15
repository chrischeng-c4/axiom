# #1550/#1582 — builtin unbound-dispatch-to-None family + introspection residuals

Status: OPEN (p2, two issues). Design for implementation.

## Mechanism

Family symptom `TypeError: 'NoneType' object is not callable` when a builtin
type's member is reached through a non-standard path — lookup returns
None, caller calls it. Three confirmed shapes: `type(None)()`-style singleton
construction (singletons_construct.py), `dict.__or__({..},{..})` explicit
unbound dunder call (#1582), multi-class composite construction (#1557's
composite probe). Likely one resolution gap: the builtin-type member table
consulted for unbound/direct access misses these entries while instance-bound
access works. DIAGNOSE the three shapes together (also see
`exceptions/1557-exc-subclass-init-shapes.md`) before writing three fixes.

Separate residuals bundled in #1550:
- `dir_introspection.py`: dir() contents diverge (runtime AssertionError) —
  note the known dir()-leaks-dict-methods bug from goal-loop wave 3 (real
  modules' dir() surfaces backing-dict method names); may be the same defect.
- `callable_broad.py`: `callable()` returns False for some callable shape —
  identify which object class; likely the same member-table gap.
- `chr/behavior.py`: compile-time `expected str, got bytes` — TYPE-SYSTEM
  false positive, fix per `1595-ingress-overwalling-shapes.md` patterns, not
  here.

## Invariant

Unbound access `BuiltinType.member` and bound access `instance.member` resolve
to the same underlying callable (modulo binding); a lookup that would return
None for a member that bound-access resolves is a defect.

## Verification contract

Fixtures: singletons_construct.py, builtin_protocol_methods.py (#1582),
dir_introspection.py, callable_broad.py — byte-identical vs oracle; the
`_regression/builtin-libs/builtins/` dir sweep FAIL count drops accordingly;
gate no worse. Cross-comment findings on #1557 if the family root is shared.
