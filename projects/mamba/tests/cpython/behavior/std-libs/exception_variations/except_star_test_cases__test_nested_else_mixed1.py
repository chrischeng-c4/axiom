# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_nested_else_mixed1"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_nested_else_mixed1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_nested_else_mixed1
"""Auto-ported test: ExceptStarTestCases::test_nested_else_mixed1 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
try:
    try:
        pass
    except* BaseException:
        hit_inner_except = True
    else:
        hit_inner_else = True
    raise Exception('outer exception')
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_inner_except

assert hit_inner_else

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptStarTestCases::test_nested_else_mixed1: ok")
