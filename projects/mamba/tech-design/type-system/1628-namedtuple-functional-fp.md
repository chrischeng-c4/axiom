# #1628 — typing.NamedTuple functional form walled

Status: OPEN (p2). Design for implementation.

## Mechanism

`NamedTuple('N', [...])` (the functional factory call) is rejected by the
checker. Direct sibling of the #1595 ClosureVars fix — that fix split
signature resolution into "reached via inheritance" (skip the
`typing.NamedTuple` factory-signature owner) vs "callee IS NamedTuple" (keep
the factory signature as a wall). The functional form is the LEGAL use of that
factory signature, so if it walls, the retained direct-call signature is
misaligned with the actual factory shape (`(typename: str, fields:
list[tuple[str, type]] | ...)`) or the call-shape classifier misroutes it.

## Invariant

The inheritance-path skip from #1595 (`is_named_tuple_base_owner`) stays; the
direct factory wall `NamedTuple__init__typename_as_str_wrong.py` stays red.
Legal functional calls (positional typename + fields list, kwargs form) pass.

## Fix direction

In check_expr's structured_stdlib_constructor path: align the direct-call
signature with the real factory (both fields-list and kwargs forms), or widen
the fields param to accept the observed legal shapes. Diagnose with the exact
fixture from #1627's sweep first — the wrong expected-type in the message
identifies which param is misaligned.

## Verification contract

The victim fixture(s) byte-identical vs oracle; the typename wall guard stays
rejected; typing-family focused sweep no regressions; gate no worse.
