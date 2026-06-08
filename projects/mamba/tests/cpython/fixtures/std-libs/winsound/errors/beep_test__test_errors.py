# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winsound"
# dimension = "errors"
# case = "beep_test__test_errors"
# subject = "cpython.test_winsound.BeepTest.test_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_winsound.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_winsound.py::BeepTest::test_errors
"""Auto-ported test: BeepTest::test_errors (CPython 3.12 oracle)."""


try:
    import winsound
except ImportError:
    print("BeepTest::test_errors: skipped, winsound unavailable")
    raise SystemExit(0)


def assert_raises(exc_type, func, *args):
    try:
        func(*args)
    except exc_type:
        return
    raise AssertionError(f"{func.__name__}{args!r} did not raise {exc_type.__name__}")


assert_raises(TypeError, winsound.Beep)
assert_raises(ValueError, winsound.Beep, 36, 75)
assert_raises(ValueError, winsound.Beep, 32768, 75)

print("BeepTest::test_errors: ok")
