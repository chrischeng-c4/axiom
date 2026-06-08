# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_super"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_super"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_super
"""Auto-ported test: PositionalOnlyTestCase::test_super (CPython 3.12 oracle)."""


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
sentinel = object()

class A:

    def method(self):
        return sentinel

class C(A):

    def method(self, /):
        return super().method()

assert C().method() == sentinel
print("PositionalOnlyTestCase::test_super: ok")
