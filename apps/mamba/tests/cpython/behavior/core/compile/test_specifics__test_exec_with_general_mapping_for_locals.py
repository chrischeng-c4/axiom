# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_exec_with_general_mapping_for_locals"
# subject = "cpython.test_compile.TestSpecifics.test_exec_with_general_mapping_for_locals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_exec_with_general_mapping_for_locals
"""Auto-ported test: TestSpecifics::test_exec_with_general_mapping_for_locals (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
class M:
    """Test mapping interface versus possible calls from eval()."""

    def __getitem__(self, key):
        if key == 'a':
            return 12
        raise KeyError

    def __setitem__(self, key, value):
        self.results = (key, value)

    def keys(self):
        return list('xyz')
m = M()
g = globals()
exec('z = a', g, m)

assert m.results == ('z', 12)
try:
    exec('z = b', g, m)
except NameError:
    pass
else:

    raise AssertionError('Did not detect a KeyError')
exec('z = dir()', g, m)

assert m.results == ('z', list('xyz'))
exec('z = globals()', g, m)

assert m.results == ('z', g)
exec('z = locals()', g, m)

assert m.results == ('z', m)

try:
    exec('z = b', m)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class A:
    """Non-mapping"""
    pass
m = A()

try:
    exec('z = a', g, m)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class D(dict):

    def __getitem__(self, key):
        if key == 'a':
            return 12
        return dict.__getitem__(self, key)
d = D()
exec('z = a', g, d)

assert d['z'] == 12
print("TestSpecifics::test_exec_with_general_mapping_for_locals: ok")
