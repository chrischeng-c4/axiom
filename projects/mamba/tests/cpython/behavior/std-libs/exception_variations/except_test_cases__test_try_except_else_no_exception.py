# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except_else_no_exception"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except_else_no_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except_else_no_exception
"""Auto-ported test: ExceptTestCases::test_try_except_else_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_else = False
try:
    pass
except:
    hit_except = True
else:
    hit_else = True

assert not hit_except

assert hit_else
print("ExceptTestCases::test_try_except_else_no_exception: ok")
