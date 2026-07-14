# #1536 — self-referential container mutation must not wall

Status: OPEN (p1). Design for implementation.

## Mechanism

Element-type inference pins `cyclic = [1, 2]` to `list[int]`, then
`cyclic.append(cyclic)` fails the element check: `expected int, got
list[int]` — a compile reject of legal Python (fixture
`behavior/std-libs/copy/deepcopy_reflexive_list_cycle.py`). Same family as
the #1550 `chr` case (`expected str, got bytes` at compile). Distinct from
#220's ==-search relaxation: `append/insert/extend` MUTATE, so the correct
response is widening the inferred element type, not bypassing the check.

## Invariant

A mutation call whose argument is (or contains) the receiver itself widens the
receiver's element type (to a union with the container type, or Any if
recursive types aren't representable) instead of erroring. Genuinely
wrong-typed scalar appends keep walling (`list__append__object_as__T_wrong`
guard stays red).

## Fix direction

In the checker's mutation-method arm (check_expr, near #220's
`container_receiver_relaxed_call` precedent): when method ∈
{append, insert, extend, add} and the value arg's type is the receiver's own
container type (or receiver expression aliases the object), record a widened
element type for subsequent reads rather than emitting the mismatch. Check
dict/set self-referential shapes (`d['self'] = d`) for the same handling.

## Verification contract

Fixture above + minimal probes (list/dict/set self-reference) byte-identical
vs python3.12; type/ dimension append/add guard fixtures still rejected;
contextvars+copy dir sweep stays at #232-era counts. Gate before/after.
