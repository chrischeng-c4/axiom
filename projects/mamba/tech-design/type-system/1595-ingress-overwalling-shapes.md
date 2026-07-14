# #1595/#1615 — strict-type ingress over-walling: legal shapes walled at compile

Status: rolling family. Landed: `35b6be9f8` (#1595, 3 shapes), `7ee9fb8e1`
(wave 1: numeric protocols + open inheritance), `d835c7b7d`/`9949dd8ab`
(wave 2: scope bindings + duck-typed closed inheritance). Remainder tracked in
#1615 (functools 9 = constructor_bypass extension) and goal-loop waves.

## Mechanism

The dynamic-call ingress/variadics enforcement (b055a1e18/f15a81220) checks
call args against signatures that are sometimes WRONG for the call shape:
inherited `__init__` resolution walking to `typing.NamedTuple`'s factory
signature; structural protocols (`SupportsIndex/SupportsFloat`) unknown to the
general compatibility engine; user classes with unresolved external bases
hard-rejected against nominal stdlib params; iterator RHS in slice assignment
required to be a static list shape.

## Invariant (the dimension rule)

Fixtures under behavior/errors/real_world/surface/_regression/security/
concurrency MUST run — a compile reject there is by definition a checker
false positive. `type/` dimension fixtures are the walls and must KEEP
rejecting. Every fix here must show the guard (`*_wrong.py`) set unweakened.

## Fix pattern

Per-shape signature alignment or principled widening at the checker — never
blanket-disable. Precedents: inheritance-only skip of the NamedTuple factory
owner; `SupportsIndex→Int|Bool` / `SupportsFloat→Int|Float|Bool` in
`types_compatible_inner`; open-inheritance leniency mirrored from
`stdlib_type_relation_inner` into the nominal catch-all; permissive deferral
for non-container slice-assign RHS. For errors/-dimension "should raise at
runtime" shapes: extend the `constructor_bypass` allowlist
(check_expr.rs ~L4313) case-by-case.

## Verification contract

Victim fixtures byte-identical vs oracle; type/ guard sweep 0 new FAIL;
full gate before/after reading in every wave report (#1615 comments).
