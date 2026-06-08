# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "behavior"
# case = "dis_tests__test_unknown_opcode"
# subject = "cpython.test_pickletools.DisTests.test_unknown_opcode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickletools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pickletools.py::DisTests::test_unknown_opcode
"""Auto-ported test: DisTests::test_unknown_opcode (CPython 3.12 oracle)."""


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
maxDiff = None

def check_dis(data, expected, **kwargs):
    out = io.StringIO()
    pickletools.dis(data, out=out, **kwargs)

    assert out.getvalue() == expected

def check_dis_error(data, expected, expected_error, **kwargs):
    out = io.StringIO()
    try:
        pickletools.dis(data, out=out, **kwargs)
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import re as _re_aR
        assert _re_aR.search(expected_error, str(_aR_e))

    assert out.getvalue() == expected
check_dis_error(b'N\xff', '    0: N    NONE\n', "at position 1, opcode b'\\\\xff' unknown")
print("DisTests::test_unknown_opcode: ok")
