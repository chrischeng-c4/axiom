# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "test_exception_ops"
# subject = "cpython321.test_exception_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_exception_ops.py"
# status = "filled"
# ///
"""cpython321.test_exception_ops: execute CPython 3.12 seed test_exception_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for exception handling semantics.
# Surface: raise+except+as, except-after-conversion-failure, finally
# always runs, try/except/finally ordering, multi-except dispatch.
# Companion to stub/test_exception.py — vendored unittest seed.
_ledger: list[int] = []

caught = ""
try:
    raise ValueError("bad")
except ValueError as e:
    caught = str(e)
assert caught == "bad"; _ledger.append(1)

n = 0
try:
    n = int("abc")
except ValueError:
    n = -1
assert n == -1; _ledger.append(1)

f = "no"
try:
    pass
finally:
    f = "yes"
assert f == "yes"; _ledger.append(1)

trace: list[str] = []
try:
    trace.append("try")
    raise RuntimeError("oops")
except RuntimeError:
    trace.append("except")
finally:
    trace.append("finally")
assert trace == ["try", "except", "finally"]; _ledger.append(1)

def classify(x: int) -> str:
    try:
        if x == 0:
            raise ZeroDivisionError("zero")
        elif x < 0:
            raise ValueError("neg")
        return "ok"
    except ZeroDivisionError:
        return "zero"
    except ValueError:
        return "neg"

assert classify(5) == "ok"; _ledger.append(1)
assert classify(0) == "zero"; _ledger.append(1)
assert classify(-3) == "neg"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_exception_ops {sum(_ledger)} asserts")
