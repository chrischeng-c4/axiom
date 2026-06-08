# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except_finally"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except_finally"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except_finally
"""Auto-ported test: ExceptStarTestCases::test_try_except_finally (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_finally = False
try:
    raise Exception('yarr!')
except* BaseException:
    hit_except = True
finally:
    hit_finally = True

assert hit_except

assert hit_finally
print("ExceptStarTestCases::test_try_except_finally: ok")
