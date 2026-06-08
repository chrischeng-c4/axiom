# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except
"""Auto-ported test: ExceptTestCases::test_try_except (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
try:
    raise Exception('ahoy!')
except:
    hit_except = True

assert hit_except
print("ExceptTestCases::test_try_except: ok")
