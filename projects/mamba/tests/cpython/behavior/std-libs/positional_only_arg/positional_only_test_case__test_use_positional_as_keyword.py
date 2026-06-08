# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_use_positional_as_keyword"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_use_positional_as_keyword"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_use_positional_as_keyword
"""Auto-ported test: PositionalOnlyTestCase::test_use_positional_as_keyword (CPython 3.12 oracle)."""


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
def f(a, /):
    pass
expected = "f\\(\\) got some positional-only arguments passed as keyword arguments: 'a'"
try:
    f(a=1)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(expected, str(_aR_e))

def f(a, /, b):
    pass
expected = "f\\(\\) got some positional-only arguments passed as keyword arguments: 'a'"
try:
    f(a=1, b=2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(expected, str(_aR_e))

def f(a, b, /):
    pass
expected = "f\\(\\) got some positional-only arguments passed as keyword arguments: 'a, b'"
try:
    f(a=1, b=2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(expected, str(_aR_e))
print("PositionalOnlyTestCase::test_use_positional_as_keyword: ok")
