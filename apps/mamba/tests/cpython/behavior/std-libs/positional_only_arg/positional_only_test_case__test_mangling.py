# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_mangling"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_mangling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_mangling
"""Auto-ported test: PositionalOnlyTestCase::test_mangling (CPython 3.12 oracle)."""


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
class X:

    def f(self, __a=42, /):
        return __a

    def f2(self, __a=42, /, __b=43):
        return (__a, __b)

    def f3(self, __a=42, /, __b=43, *, __c=44):
        return (__a, __b, __c)

assert X().f() == 42

assert X().f2() == (42, 43)

assert X().f3() == (42, 43, 44)
print("PositionalOnlyTestCase::test_mangling: ok")
