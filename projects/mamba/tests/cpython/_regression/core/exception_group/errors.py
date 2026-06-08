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


# Leaves wrapped into an ExceptionGroup keep their own __context__/__cause__.
def build_nested_group():
    excs = []
    try:
        try:
            raise TypeError("ctx")
        except TypeError as e:
            raise ExceptionGroup("nested", [e])
    except ExceptionGroup as e:
        excs.append(e)
    try:
        try:
            raise MemoryError("cause")
        except MemoryError as e:
            raise ValueError(1) from e
    except ValueError as e:
        excs.append(e)
    try:
        raise ExceptionGroup("root", excs)
    except ExceptionGroup as eg:
        return eg


grp = build_nested_group()
assert isinstance(grp.exceptions[1].__context__, MemoryError)
assert isinstance(grp.exceptions[1].__cause__, MemoryError)
assert isinstance(grp.exceptions[0].__context__, TypeError)
print("group_leaf_chaining: context+cause preserved")
