# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_positional_only_with_optional_invalid_calls"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_positional_only_with_optional_invalid_calls"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_positional_only_with_optional_invalid_calls
"""Auto-ported test: PositionalOnlyTestCase::test_positional_only_with_optional_invalid_calls (CPython 3.12 oracle)."""


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
def f(a, b=2, /):
    pass
f(1)
try:
    f()
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search("f\\(\\) missing 1 required positional argument: 'a'", str(_aR_e))
try:
    f(1, 2, 3)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('f\\(\\) takes from 1 to 2 positional arguments but 3 were given', str(_aR_e))
print("PositionalOnlyTestCase::test_positional_only_with_optional_invalid_calls: ok")
