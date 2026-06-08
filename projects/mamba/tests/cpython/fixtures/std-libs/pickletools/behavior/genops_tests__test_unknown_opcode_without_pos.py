# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "behavior"
# case = "genops_tests__test_unknown_opcode_without_pos"
# subject = "cpython.test_pickletools.GenopsTests.test_unknown_opcode_without_pos"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickletools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pickletools.py::GenopsTests::test_unknown_opcode_without_pos
"""Auto-ported test: GenopsTests::test_unknown_opcode_without_pos (CPython 3.12 oracle)."""


import io
import pickle
import pickletools
from test import support
from test.pickletester import AbstractPickleTests
import doctest
import unittest


class SimpleReader:

    def __init__(self, data):
        self.data = data
        self.pos = 0

    def read(self, n):
        data = self.data[self.pos:self.pos + n]
        self.pos += n
        return data

    def readline(self):
        nl = self.data.find(b'\n', self.pos) + 1
        if not nl:
            nl = len(self.data)
        data = self.data[self.pos:nl]
        self.pos = nl
        return data

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(pickletools))
    return tests


# --- test body ---
f = SimpleReader(b'N\xff')
it = pickletools.genops(f)
item = next(it)

assert item[0].name == 'NONE'
try:
    next(it)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search("at position <unknown>, opcode b'\\\\xff' unknown", str(_aR_e))
print("GenopsTests::test_unknown_opcode_without_pos: ok")
