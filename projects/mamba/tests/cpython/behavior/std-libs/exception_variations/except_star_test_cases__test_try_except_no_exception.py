# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except_no_exception"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except_no_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except_no_exception
"""Auto-ported test: ExceptStarTestCases::test_try_except_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
try:
    pass
except* BaseException:
    hit_except = True

assert not hit_except
print("ExceptStarTestCases::test_try_except_no_exception: ok")
