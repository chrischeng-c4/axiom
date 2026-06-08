# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_annotations_constant_fold"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_annotations_constant_fold"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_positional_only_arg.py::PositionalOnlyTestCase::test_annotations_constant_fold
"""Auto-ported test: PositionalOnlyTestCase::test_annotations_constant_fold (CPython 3.12 oracle)."""


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
def g():

    def f(x: not int is int, /):
        ...
codes = [(i.opname, i.argval) for i in dis.get_instructions(g)]

assert ('UNARY_NOT', None) not in codes

assert ('IS_OP', 1) in codes
print("PositionalOnlyTestCase::test_annotations_constant_fold: ok")
