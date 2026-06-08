# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "grammar"
# dimension = "behavior"
# case = "grammar_tests__test_shift_ops"
# subject = "cpython.test_grammar.GrammarTests.test_shift_ops"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_grammar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_grammar.py::GrammarTests::test_shift_ops
"""Auto-ported test: GrammarTests::test_shift_ops (CPython 3.12 oracle)."""


from test.support import check_syntax_error
from test.support import import_helper
import inspect
import unittest
import sys
import warnings
from sys import *
import test.typinganndata.ann_module as ann_module
import typing
from test.typinganndata import ann_module2
import test


VALID_UNDERSCORE_LITERALS = ['0_0_0', '4_2', '1_0000_0000', '0b1001_0100', '0xffff_ffff', '0o5_7_7', '1_00_00.5', '1_00_00.5e5', '1_00_00e5_1', '1e1_0', '.1_4', '.1_4e1', '0b_0', '0x_f', '0o_5', '1_00_00j', '1_00_00.5j', '1_00_00e5_1j', '.1_4j', '(1_2.5+3_3j)', '(.5_6j)']

INVALID_UNDERSCORE_LITERALS = ['0_', '42_', '1.4j_', '0x_', '0b1_', '0xf_', '0o5_', '0 if 1_Else 1', '0_b0', '0_xf', '0_o5', '0_7', '09_99', '4_______2', '0.1__4', '0.1__4j', '0b1001__0100', '0xffff__ffff', '0x___', '0o5__77', '1e1__0', '1e1__0j', '1_.4', '1_.4j', '1._4', '1._4j', '._5', '._5j', '1.0e+_1', '1.0e+_1j', '1.4_j', '1.4e5_j', '1_e1', '1.4_e1', '1.4_e1j', '1e_1', '1.4e_1', '1.4e_1j', '(1+1.5_j_)', '(1+1.5_j)']

var_annot_global: int

class CNS:

    def __init__(self):
        self._dct = {}

    def __setitem__(self, item, value):
        self._dct[item.lower()] = value

    def __getitem__(self, item):
        return self._dct[item]


# --- test body ---
x = 1 << 1
x = 1 >> 1
x = 1 << 1 >> 1
print("GrammarTests::test_shift_ops: ok")
