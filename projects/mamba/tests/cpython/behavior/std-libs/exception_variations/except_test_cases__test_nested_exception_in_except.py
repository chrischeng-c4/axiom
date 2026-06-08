# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_nested_exception_in_except"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_nested_exception_in_except"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_nested_exception_in_except
"""Auto-ported test: ExceptTestCases::test_nested_exception_in_except (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
try:
    try:
        raise Exception('inner exception')
    except:
        hit_inner_except = True
        raise Exception('outer exception')
    else:
        hit_inner_else = True
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert hit_inner_except

assert not hit_inner_else

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptTestCases::test_nested_exception_in_except: ok")
