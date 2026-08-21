# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Context manager — __enter__ / __exit__ — #2790.
#
# Covers Python's `with` statement context-manager protocol:
#
#   __enter__   called when the with block starts. Returns the
#               value bound to `as`.
#   __exit__    called when the block exits — normally OR via
#               exception. Receives (exc_type, exc, tb); returning
#               a truthy value SUPPRESSES the exception.
#
# Clauses:
#   1. Normal path — __enter__ runs, block runs, __exit__ runs with
#      (None, None, None).
#   2. Exception path — __enter__ runs, exception inside the block,
#      __exit__ receives the exception triple; if __exit__ returns
#      falsy the exception propagates.
#   3. Suppression path — __exit__ returns True; the exception does
#      NOT propagate out of the with block.
#   4. `as` binding — the value bound after `as` is whatever
#      __enter__ returns (NOT the context-manager object unless
#      __enter__ returns `self`).
#   5. __exit__ runs even if the block returns / breaks / continues
#      early. We use `return` from inside a function to prove
#      __exit__ still fires.
#   6. Multiple with clauses on one line nest correctly: outer
#      __enter__, inner __enter__, body, inner __exit__, outer
#      __exit__.
#
# Every print line tagged `[ctx-mgr]` so failure output names
# context-manager semantics.


TRACE = []


class CM:
    def __init__(self, label, suppress=False, enter_value=None):
        self.label = label
        self.suppress = suppress
        self.enter_value = enter_value if enter_value is not None else self
        self.exit_received = None

    def __enter__(self):
        TRACE.append(f"enter:{self.label}")
        return self.enter_value

    def __exit__(self, exc_type, exc, tb):
        TRACE.append(f"exit:{self.label}")
        self.exit_received = (
            exc_type.__name__ if exc_type is not None else None,
            str(exc) if exc is not None else None,
            tb is not None,
        )
        return self.suppress


# Clause 1: normal path.
TRACE.clear()
cm = CM("c1")
with cm:
    TRACE.append("body:c1")
print("[ctx-mgr] clause-1 trace:", TRACE[:])
print("[ctx-mgr] clause-1 exit-received:", cm.exit_received)


# Clause 2: exception propagates when __exit__ returns falsy.
TRACE.clear()
cm = CM("c2", suppress=False)
try:
    with cm:
        TRACE.append("body:c2-before-raise")
        raise ValueError("boom")
except ValueError as exc:
    TRACE.append(f"caught:{exc}")
print("[ctx-mgr] clause-2 trace:", TRACE[:])
print("[ctx-mgr] clause-2 exit-received:", cm.exit_received)


# Clause 3: suppression — __exit__ returns True.
TRACE.clear()
cm = CM("c3", suppress=True)
with cm:
    TRACE.append("body:c3-before-raise")
    raise ValueError("suppressed")
TRACE.append("after-with-c3")
print("[ctx-mgr] clause-3 trace:", TRACE[:])
print("[ctx-mgr] clause-3 exit-received:", cm.exit_received)


# Clause 4: as-binding receives __enter__'s return value.
TRACE.clear()
sentinel = object()
cm = CM("c4", enter_value="enter-returned")
with cm as v:
    print("[ctx-mgr] clause-4 as-value:", v)

cm2 = CM("c4-self")  # enter_value defaults to self
with cm2 as v:
    print("[ctx-mgr] clause-4 as-is-self:", v is cm2)

# Suppress unused-sentinel warning by referencing it.
_ = sentinel


# Clause 5: __exit__ runs on early return from a function.
TRACE.clear()


def fn():
    cm = CM("c5")
    with cm:
        TRACE.append("body:c5-before-return")
        return "function-return"


fn_result = fn()
print("[ctx-mgr] clause-5 result:", fn_result)
print("[ctx-mgr] clause-5 trace:", TRACE[:])


# Clause 6: nested with statements.
TRACE.clear()
with CM("outer"), CM("inner"):
    TRACE.append("body:nested")
print("[ctx-mgr] clause-6 trace:", TRACE[:])
