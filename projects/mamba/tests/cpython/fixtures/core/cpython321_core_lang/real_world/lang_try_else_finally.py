# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_try_else_finally"
# subject = "cpython321.lang_try_else_finally"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_try_else_finally.py"
# status = "filled"
# ///
"""cpython321.lang_try_else_finally: execute CPython 3.12 seed lang_try_else_finally"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for advanced try/except/else/finally
# control-flow shapes.
# Surface: try-else (runs only on no-exception success path), tuple
# except clause (catches any of the listed types), bare `raise` to
# re-throw the current exception in an outer handler, finally runs
# even when the try body returns early.
_ledger: list[int] = []

# Success path: try, then else, then finally. No exception raised.
trace1: list[str] = []
try:
    trace1.append("try")
except ValueError:
    trace1.append("except")
else:
    trace1.append("else")
finally:
    trace1.append("finally")
assert trace1 == ["try", "else", "finally"]; _ledger.append(1)

# Exception path: try, then except, then finally — `else` is SKIPPED
trace2: list[str] = []
try:
    trace2.append("try")
    raise ValueError("v")
except ValueError:
    trace2.append("except")
else:
    trace2.append("else")
finally:
    trace2.append("finally")
assert trace2 == ["try", "except", "finally"]; _ledger.append(1)

# Tuple except — first matching clause wins; the exception type is in
# the tuple
trace3: list[str] = []
try:
    raise KeyError("k")
except (TypeError, KeyError):
    trace3.append("typekey")
except Exception:
    trace3.append("exc")
assert trace3 == ["typekey"]; _ledger.append(1)

# Re-raise propagates the same exception out to the next handler
trace4: list[str] = []
def outer():
    try:
        try:
            raise ValueError("inner")
        except ValueError:
            trace4.append("caught")
            raise
    except ValueError as e:
        trace4.append(f"outer:{e}")

outer()
assert trace4 == ["caught", "outer:inner"]; _ledger.append(1)

# finally runs even when the try body executes `return`
trace5: list[str] = []
def with_finally():
    try:
        return "early"
    finally:
        trace5.append("late")

r = with_finally()
assert r == "early"; _ledger.append(1)
assert trace5 == ["late"]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_try_else_finally {sum(_ledger)} asserts")
