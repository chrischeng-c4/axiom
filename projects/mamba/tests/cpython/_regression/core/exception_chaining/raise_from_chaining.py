# raise-from exception chaining — #2795.
#
# Covers Python's exception chaining attributes:
#
#   exc.__cause__              the explicit cause set by `raise X from Y`.
#                              Default None.
#   exc.__context__            the implicit context — the exception
#                              that was being handled when `raise`
#                              occurred. Set automatically.
#   exc.__suppress_context__   True when `raise X from Y` or
#                              `raise X from None` is used; the
#                              traceback printer hides __context__
#                              under that flag.
#
# Clauses:
#   1. `raise X from Y` — __cause__ is Y, __context__ is the active
#      exception, __suppress_context__ is True.
#   2. Bare `raise X` inside `except` — __cause__ is None,
#      __context__ is the original exception, __suppress_context__
#      is False (the printer shows "During handling of...").
#   3. `raise X from None` — __cause__ is None,
#      __suppress_context__ is True. The previous-exception context
#      is hidden by the traceback formatter.
#   4. No active exception — `raise X` standalone: __cause__ is None,
#      __context__ is None, __suppress_context__ is False.
#   5. Chained chain — `try-except-raise-from` nested inside another
#      `try-except-raise-from` walks the __cause__ chain to depth 2.
#   6. __cause__ accepts any BaseException, NOT only Exception (we
#      use a KeyboardInterrupt instance to prove it).
#
# Every print line tagged `[chaining]` so failure output names
# exception-chaining semantics.


# Clause 1: explicit from.
caught1 = None
try:
    try:
        raise KeyError("k-orig")
    except KeyError as orig:
        raise ValueError("v-new") from orig
except ValueError as exc:
    caught1 = exc

print(
    "[chaining] clause-1 cause-type:",
    type(caught1.__cause__).__name__ if caught1 is not None else None,
)
print(
    "[chaining] clause-1 context-type:",
    type(caught1.__context__).__name__ if caught1 is not None else None,
)
print(
    "[chaining] clause-1 suppress-context:",
    caught1.__suppress_context__ if caught1 is not None else None,
)


# Clause 2: bare raise inside except — implicit context only.
caught2 = None
try:
    try:
        raise KeyError("k-orig")
    except KeyError:
        raise ValueError("v-new")
except ValueError as exc:
    caught2 = exc

print(
    "[chaining] clause-2 cause-is-none:",
    caught2.__cause__ is None if caught2 is not None else None,
)
print(
    "[chaining] clause-2 context-type:",
    type(caught2.__context__).__name__ if caught2 is not None else None,
)
print(
    "[chaining] clause-2 suppress-context:",
    caught2.__suppress_context__ if caught2 is not None else None,
)


# Clause 3: `raise X from None` — suppress the chain.
caught3 = None
try:
    try:
        raise KeyError("k-orig")
    except KeyError:
        raise ValueError("v-new") from None
except ValueError as exc:
    caught3 = exc

print(
    "[chaining] clause-3 cause-is-none:",
    caught3.__cause__ is None if caught3 is not None else None,
)
print(
    "[chaining] clause-3 suppress-context:",
    caught3.__suppress_context__ if caught3 is not None else None,
)
# __context__ is still the original — only suppression flag changed.
print(
    "[chaining] clause-3 context-type:",
    type(caught3.__context__).__name__ if caught3 is not None else None,
)


# Clause 4: no active exception.
caught4 = None
try:
    raise ValueError("standalone")
except ValueError as exc:
    caught4 = exc

print(
    "[chaining] clause-4 cause-is-none:",
    caught4.__cause__ is None if caught4 is not None else None,
)
print(
    "[chaining] clause-4 context-is-none:",
    caught4.__context__ is None if caught4 is not None else None,
)
print(
    "[chaining] clause-4 suppress-context:",
    caught4.__suppress_context__ if caught4 is not None else None,
)


# Clause 5: depth-2 chain walking.
def walk_causes(exc):
    out = []
    cur = exc
    while cur is not None:
        out.append(type(cur).__name__)
        cur = cur.__cause__
    return out


caught5 = None
try:
    try:
        try:
            raise KeyError("level-0")
        except KeyError as orig0:
            raise LookupError("level-1") from orig0
    except LookupError as orig1:
        raise ValueError("level-2") from orig1
except ValueError as exc:
    caught5 = exc

print("[chaining] clause-5 cause-chain:", walk_causes(caught5))


# Clause 6: __cause__ accepts any BaseException, not only Exception.
caught6 = None
try:
    try:
        raise KeyboardInterrupt()
    except KeyboardInterrupt as kb:
        raise ValueError("wrap") from kb
except ValueError as exc:
    caught6 = exc

print(
    "[chaining] clause-6 cause-base-type:",
    type(caught6.__cause__).__name__ if caught6 is not None else None,
)
print(
    "[chaining] clause-6 cause-is-base-exception:",
    isinstance(caught6.__cause__, BaseException) if caught6 is not None else None,
)
