# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_ensure_flags_dont_clash"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_ensure_flags_dont_clash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_ensure_flags_dont_clash
"""Auto-ported test: FutureTest::test_ensure_flags_dont_clash (CPython 3.12 oracle)."""


import __future__
import ast


flags = {
    f"CO_FUTURE_{future.upper()}": getattr(__future__, future).compiler_flag
    for future in __future__.all_feature_names
}
flags |= {
    flag: getattr(ast, flag)
    for flag in dir(ast)
    if flag.startswith("PyCF_")
}

values = list(flags.values())
assert len(set(values)) == len(values), flags

print("FutureTest::test_ensure_flags_dont_clash: ok")
