# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_pos_only_definition"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_pos_only_definition"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_pos_only_definition
"""Auto-ported test: PositionalOnlyTestCase::test_pos_only_definition (CPython 3.12 oracle)."""


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
def f(a, b, c, /, d, e=1, *, f, g=2):
    pass

assert 5 == f.__code__.co_argcount

assert 3 == f.__code__.co_posonlyargcount

assert (1,) == f.__defaults__

def f(a, b, c=1, /, d=2, e=3, *, f, g=4):
    pass

assert 5 == f.__code__.co_argcount

assert 3 == f.__code__.co_posonlyargcount

assert (1, 2, 3) == f.__defaults__
print("PositionalOnlyTestCase::test_pos_only_definition: ok")
