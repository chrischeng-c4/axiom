# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_same_keyword_as_positional_with_kwargs"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_same_keyword_as_positional_with_kwargs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_same_keyword_as_positional_with_kwargs
"""Auto-ported test: PositionalOnlyTestCase::test_same_keyword_as_positional_with_kwargs (CPython 3.12 oracle)."""


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
def f(something, /, **kwargs):
    return (something, kwargs)

assert f(42, something=42) == (42, {'something': 42})
try:
    f(something=42)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search("f\\(\\) missing 1 required positional argument: 'something'", str(_aR_e))

assert f(42) == (42, {})
print("PositionalOnlyTestCase::test_same_keyword_as_positional_with_kwargs: ok")
