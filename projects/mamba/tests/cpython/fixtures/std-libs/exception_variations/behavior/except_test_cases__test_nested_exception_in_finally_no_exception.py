# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_nested_exception_in_finally_no_exception"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_nested_exception_in_finally_no_exception"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_nested_exception_in_finally_no_exception
"""Auto-ported test: ExceptTestCases::test_nested_exception_in_finally_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
hit_inner_finally = False
try:
    try:
        pass
    except:
        hit_inner_except = True
    else:
        hit_inner_else = True
    finally:
        hit_inner_finally = True
        raise Exception('outer exception')
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_inner_except

assert hit_inner_else

assert hit_inner_finally

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptTestCases::test_nested_exception_in_finally_no_exception: ok")
