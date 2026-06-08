# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_nested"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_nested"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_nested
"""Auto-ported test: ExceptTestCases::test_nested (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_finally = False
hit_inner_except = False
hit_inner_finally = False
try:
    try:
        raise Exception('inner exception')
    except:
        hit_inner_except = True
    finally:
        hit_inner_finally = True
finally:
    hit_finally = True

assert hit_inner_except

assert hit_inner_finally

assert hit_finally
print("ExceptTestCases::test_nested: ok")
