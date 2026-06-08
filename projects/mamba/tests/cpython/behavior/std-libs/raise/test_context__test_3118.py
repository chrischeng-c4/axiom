# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_context__test_3118"
# subject = "cpython.test_raise.TestContext.test_3118"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestContext::test_3118 (CPython 3.12 oracle)."""


def gen():
    try:
        yield 1
    finally:
        pass


g = gen()
assert next(g) == 1

try:
    try:
        raise ValueError
    except Exception:
        del g
        raise KeyError
except Exception as exc:
    assert isinstance(exc, KeyError), type(exc)
    assert isinstance(exc.__context__, ValueError), exc.__context__
else:
    raise AssertionError("expected KeyError with preserved ValueError context")

print("TestContext::test_3118: ok")
