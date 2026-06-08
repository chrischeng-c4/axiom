# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except_finally"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except_finally"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except_finally
"""Auto-ported test: ExceptTestCases::test_try_except_finally (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_finally = False
try:
    raise Exception('yarr!')
except:
    hit_except = True
finally:
    hit_finally = True

assert hit_except

assert hit_finally
print("ExceptTestCases::test_try_except_finally: ok")
