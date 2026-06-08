# try / except / else / finally ordering — #2794.
#
# Covers the execution sequence of Python's exception-control-flow
# clauses across the four canonical paths:
#
#   normal:    try-body -> else -> finally
#   handled:   try-body -> except -> finally
#   unhandled: try-body -> finally -> propagation
#   return:    try-body (with return) -> finally; finally can SEE the
#              pending return value, and a return in finally REPLACES
#              the pending return.
#
# Clauses:
#   1. Normal — `else` runs only when try-body completed without
#      raising; finally runs last.
#   2. Handled — except runs, else does NOT, finally runs last.
#   3. Unhandled — finally runs THEN exception propagates.
#   4. Return-in-try — finally runs before the function actually
#      returns; the return value is preserved unless finally also
#      returns.
#   5. Return-in-finally — overrides the try-body return.
#   6. break / continue in finally — Python permits break inside
#      finally; we exercise the loop-control interaction.
#
# Every print line tagged `[control-flow]` so failure output names
# exception-control-flow semantics.


TRACE = []


def case_normal():
    TRACE.clear()
    try:
        TRACE.append("try")
    except Exception:  # pyright: ignore[reportUnusedExcept]
        TRACE.append("except")
    else:
        TRACE.append("else")
    finally:
        TRACE.append("finally")
    return TRACE[:]


from typing import Any


def _maybe(exc: Exception) -> Any:
    """Identity wrapper that erases the type for pyright."""
    return exc


def case_handled():
    TRACE.clear()
    raised = _maybe(ValueError("handled"))
    try:
        TRACE.append("try-before-raise")
        if raised is not None:
            raise raised
    except ValueError:
        TRACE.append("except")
    else:
        TRACE.append("else-unreached")
    finally:
        TRACE.append("finally")
    return TRACE[:]


def case_unhandled():
    TRACE.clear()
    raised = _maybe(RuntimeError("unhandled"))
    try:
        try:
            TRACE.append("try-before-raise")
            if raised is not None:
                raise raised
        except ValueError:
            TRACE.append("except-wrong-type")
        else:
            TRACE.append("else-unreached")
        finally:
            TRACE.append("finally")
    except RuntimeError as exc:
        TRACE.append(f"propagated:{exc}")
    return TRACE[:]


def case_return_in_try():
    TRACE.clear()

    def inner():
        try:
            TRACE.append("try-before-return")
            return "try-return"
        finally:
            TRACE.append("finally")

    result = inner()
    return TRACE[:], result


# `return` and `break` inside `finally` are valid in Python 3.12 but
# pyright (and PEP 765 in 3.14) flags them. Compile via exec() so the
# static check sees only the call site.
_RETURN_IN_FINALLY_SRC = """
def inner():
    try:
        TRACE.append("try-before-return")
        return "try-return"
    finally:
        TRACE.append("finally-overrides")
        return "finally-return"
result = inner()
"""

_BREAK_IN_FINALLY_SRC = """
for i in range(3):
    try:
        TRACE.append(f"try:{i}")
        if i == 1:
            raise ValueError("trigger")
    except ValueError:
        TRACE.append(f"except:{i}")
    finally:
        TRACE.append(f"finally:{i}")
        if i == 1:
            break
TRACE.append("after-loop")
"""


def case_return_in_finally():
    TRACE.clear()
    ns: dict = {"TRACE": TRACE}
    exec(_RETURN_IN_FINALLY_SRC, ns)
    return TRACE[:], ns["result"]


def case_break_in_finally():
    TRACE.clear()
    ns: dict = {"TRACE": TRACE}
    exec(_BREAK_IN_FINALLY_SRC, ns)
    return TRACE[:]


# Clause 1: normal path.
print("[control-flow] clause-1 normal:", case_normal())


# Clause 2: handled exception.
print("[control-flow] clause-2 handled:", case_handled())


# Clause 3: unhandled exception propagates after finally.
print("[control-flow] clause-3 unhandled:", case_unhandled())


# Clause 4: return inside try — finally runs, return value preserved.
trace4, val4 = case_return_in_try()
print("[control-flow] clause-4 trace:", trace4)
print("[control-flow] clause-4 return:", val4)


# Clause 5: return inside finally overrides the try-body return.
trace5, val5 = case_return_in_finally()
print("[control-flow] clause-5 trace:", trace5)
print("[control-flow] clause-5 return:", val5)


# Clause 6: break in finally exits the surrounding loop.
print("[control-flow] clause-6 break:", case_break_in_finally())
