# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_serialization"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_serialization"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_serialization
"""Auto-ported test: PositionalOnlyTestCase::test_serialization (CPython 3.12 oracle)."""


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
pickled_posonly = pickle.dumps(global_pos_only_f)
pickled_optional = pickle.dumps(global_pos_only_and_normal)
pickled_defaults = pickle.dumps(global_pos_only_defaults)
unpickled_posonly = pickle.loads(pickled_posonly)
unpickled_optional = pickle.loads(pickled_optional)
unpickled_defaults = pickle.loads(pickled_defaults)

assert unpickled_posonly(1, 2) == (1, 2)
expected = "global_pos_only_f\\(\\) got some positional-only arguments passed as keyword arguments: 'a, b'"
try:
    unpickled_posonly(a=1, b=2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(expected, str(_aR_e))

assert unpickled_optional(1, 2) == (1, 2)
expected = "global_pos_only_and_normal\\(\\) got some positional-only arguments passed as keyword arguments: 'a'"
try:
    unpickled_optional(a=1, b=2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(expected, str(_aR_e))

assert unpickled_defaults() == (1, 2)
expected = "global_pos_only_defaults\\(\\) got some positional-only arguments passed as keyword arguments: 'a'"
try:
    unpickled_defaults(a=1, b=2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(expected, str(_aR_e))
print("PositionalOnlyTestCase::test_serialization: ok")
