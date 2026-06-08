# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/<area>: exception machinery error paths (CPython 3.12 oracle)."""


# Chained: raise X from Y sets __cause__.
try:
    try:
        raise ValueError("inner")
    except ValueError as e:
        raise RuntimeError("outer") from e
except RuntimeError as e:
    print("cause:", type(e.__cause__).__name__)
    print("ctx:", type(e.__context__).__name__)


# Bare except: catches BaseException-derived but not bareword `not exc`.
try:
    raise SystemExit("forced")
except SystemExit as e:
    print("system_exit:", str(e)[:60])


# ExceptionGroup with `except*` (PEP 654).
try:
    raise ExceptionGroup("group", [ValueError("a"), TypeError("b")])
except* ValueError as g:
    print("group_value: count=", len(g.exceptions))
except* TypeError as g:
    print("group_type: count=", len(g.exceptions))


# Re-raise preserves traceback.
try:
    try:
        raise KeyError("k")
    except KeyError:
        raise
except KeyError as e:
    print("reraised:", type(e).__name__, str(e)[:40])


# A non-class except matcher raises TypeError, not the original exception.
try:
    try:
        raise ValueError
    except 42:
        pass
    raise AssertionError("expected TypeError")
except TypeError as e:
    print("bad_matcher_scalar:", str(e)[:50])

# A non-class entry inside an except tuple is rejected the same way.
try:
    try:
        raise ValueError
    except (ValueError, 42):
        pass
    raise AssertionError("expected TypeError")
except TypeError as e:
    print("bad_matcher_tuple:", str(e)[:50])
