# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_posonly_methods"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_posonly_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_posonly_methods
"""Auto-ported test: PositionalOnlyTestCase::test_posonly_methods (CPython 3.12 oracle)."""


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
class Example:

    def f(self, a, b, /):
        return (a, b)

assert Example().f(1, 2) == (1, 2)

assert Example.f(Example(), 1, 2) == (1, 2)

try:
    Example.f(1, 2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
expected = "f\\(\\) got some positional-only arguments passed as keyword arguments: 'b'"
try:
    Example().f(1, b=2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(expected, str(_aR_e))
print("PositionalOnlyTestCase::test_posonly_methods: ok")
