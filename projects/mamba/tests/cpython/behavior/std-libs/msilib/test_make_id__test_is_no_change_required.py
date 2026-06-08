# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msilib"
# dimension = "behavior"
# case = "test_make_id__test_is_no_change_required"
# subject = "cpython.test_msilib.Test_make_id.test_is_no_change_required"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_msilib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_msilib.py::Test_make_id::test_is_no_change_required
"""Auto-ported test: Test_make_id::test_is_no_change_required (CPython 3.12 oracle)."""


try:
    import msilib
except ImportError:
    print("Test_make_id::test_is_no_change_required: skipped, msilib unavailable")
    raise SystemExit(0)


assert msilib.make_id("short") == "short"
assert msilib.make_id("nochangerequired") == "nochangerequired"
assert msilib.make_id("one.dot") == "one.dot"
assert msilib.make_id("_") == "_"
assert msilib.make_id("a") == "a"

print("Test_make_id::test_is_no_change_required: ok")
