# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# ExceptionGroup + except* — #2792.
#
# Covers Python 3.11+ PEP 654 ExceptionGroup semantics:
#
#   ExceptionGroup(msg, [exc1, exc2, ...])
#       Wraps multiple exceptions raised concurrently. `.exceptions`
#       is the sequence of leaves.
#   except* TypeRef:
#       Selects only the leaves matching `TypeRef`. Splits the group:
#       handled leaves go into the handler; unhandled leaves are
#       reraised as a residual ExceptionGroup.
#   group.split(predicate):
#       Programmatic equivalent — returns (matching_group, residual_group).
#   group.subgroup(predicate):
#       Returns ONLY the matching group (or None).
#
# This fixture is the SMOKE coverage for the language semantics
# suite. It is not a complete PEP 654 conformance run — only the
# acceptance points pinned by #2792.
#
# Clauses:
#   1. ExceptionGroup construction — type is `BaseExceptionGroup` /
#      `ExceptionGroup`, `.exceptions` round-trips the leaves.
#   2. `except* ValueError` handles ONLY ValueErrors; a residual
#      ExceptionGroup carries the rest.
#   3. `except* KeyError` matches a leaf that's nested in a sub-group.
#   4. group.split(predicate) returns the (matched, residual) pair
#      explicitly.
#   5. group.subgroup(predicate) returns matching-only group (or None
#      when nothing matches).
#   6. Unmatched-leaves residual is itself an ExceptionGroup whose
#      `.exceptions` contains the originals.
#
# Every print line tagged `[exception-group]` so failure output names
# PEP 654 semantics. Status NOT pre-marked xfail — if the runtime
# does not support ExceptionGroup the fixture fails loudly, which is
# the desired non-silent signal per the acceptance text.


# Clause 1: construction.
leaves = [ValueError("v1"), KeyError("k1"), TypeError("t1")]
group = ExceptionGroup("smoke", leaves)
print("[exception-group] clause-1 type:", type(group).__name__)
print("[exception-group] clause-1 msg:", group.message)
print("[exception-group] clause-1 leaf-count:", len(group.exceptions))
print(
    "[exception-group] clause-1 leaf-types:",
    [type(e).__name__ for e in group.exceptions],
)


# Clause 2: except* ValueError splits — handler sees ONLY ValueErrors,
# residual ExceptionGroup carries the rest. We re-raise the residual
# from another try to capture it (else clause would mask).
handled = None
residual = None
try:
    try:
        raise ExceptionGroup("split-vs", leaves)
    except* ValueError as eg:
        handled = eg
        # Re-raise everything else: the unhandled residual is
        # implicit. We don't re-raise here; the outer try gets the
        # residual via the implicit reraise.
except ExceptionGroup as eg:
    residual = eg

print(
    "[exception-group] clause-2 handled-types:",
    [type(e).__name__ for e in handled.exceptions] if handled is not None else None,
)
print(
    "[exception-group] clause-2 residual-types:",
    [type(e).__name__ for e in residual.exceptions] if residual is not None else None,
)


# Clause 3: except* matches a leaf nested in a sub-group.
nested = ExceptionGroup(
    "outer",
    [
        ValueError("outer-v"),
        ExceptionGroup("inner", [KeyError("inner-k"), TypeError("inner-t")]),
    ],
)
handled3 = None
try:
    try:
        raise nested
    except* KeyError as eg:
        handled3 = eg
except ExceptionGroup:
    pass

print(
    "[exception-group] clause-3 nested-handled-types:",
    [type(e).__name__ for e in handled3.exceptions] if handled3 is not None else None,
)


# Clause 4: group.split(predicate) — explicit programmatic split.
split_group = ExceptionGroup("split-api", leaves)
matched, residual4 = split_group.split(lambda e: isinstance(e, ValueError))
print(
    "[exception-group] clause-4 matched-types:",
    [type(e).__name__ for e in matched.exceptions] if matched is not None else None,
)
print(
    "[exception-group] clause-4 residual-types:",
    [type(e).__name__ for e in residual4.exceptions] if residual4 is not None else None,
)


# Clause 5: group.subgroup(predicate) — match-only flavor; None when
# nothing matches.
sub_match = split_group.subgroup(lambda e: isinstance(e, ValueError))
sub_empty = split_group.subgroup(lambda e: isinstance(e, RuntimeError))
print(
    "[exception-group] clause-5 sub-match-types:",
    [type(e).__name__ for e in sub_match.exceptions] if sub_match is not None else None,
)
print("[exception-group] clause-5 sub-empty-is-none:", sub_empty is None)


# Clause 6: residual is itself an ExceptionGroup with original leaves.
print(
    "[exception-group] clause-6 residual-is-group:",
    isinstance(residual, ExceptionGroup) if residual is not None else False,
)
print(
    "[exception-group] clause-6 residual-leaf-msgs:",
    [str(e) for e in residual.exceptions] if residual is not None else None,
)
