# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_no_standard_args_usage"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_no_standard_args_usage"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_no_standard_args_usage
"""Auto-ported test: PositionalOnlyTestCase::test_no_standard_args_usage (CPython 3.12 oracle)."""


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
def f(a, b, /, *, c):
    pass
f(1, 2, c=3)
try:
    f(1, b=2, c=3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PositionalOnlyTestCase::test_no_standard_args_usage: ok")
