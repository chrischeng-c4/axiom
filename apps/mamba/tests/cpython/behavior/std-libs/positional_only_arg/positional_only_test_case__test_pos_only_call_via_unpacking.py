# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_pos_only_call_via_unpacking"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_pos_only_call_via_unpacking"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_pos_only_call_via_unpacking
"""Auto-ported test: PositionalOnlyTestCase::test_pos_only_call_via_unpacking (CPython 3.12 oracle)."""


import dis
import pickle
import unittest
from test.support import check_syntax_error


'Unit tests for the positional only argument syntax specified in PEP 570.'

def global_pos_only_f(a, b, /):
    return (a, b)

def global_pos_only_and_normal(a, /, b):
    return (a, b)

def global_pos_only_defaults(a=1, /, b=2):
    return (a, b)


# --- test body ---
def f(a, b, /):
    return a + b

assert f(*[1, 2]) == 3
print("PositionalOnlyTestCase::test_pos_only_call_via_unpacking: ok")
