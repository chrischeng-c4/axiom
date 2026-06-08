# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "behavior"
# case = "dis_tests__test_annotate"
# subject = "cpython.test_pickletools.DisTests.test_annotate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickletools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pickletools.py::DisTests::test_annotate
"""Auto-ported test: DisTests::test_annotate (CPython 3.12 oracle)."""


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
check_dis(b'(Nt.', '    0: (    MARK Push markobject onto the stack.\n    1: N        NONE Push None on the stack.\n    2: t        TUPLE      (MARK at 0) Build a tuple out of the topmost stack slice, after markobject.\n    3: .    STOP                       Stop the unpickling machine.\nhighest protocol among opcodes = 0\n', annotate=1)
check_dis(b'(Nt.', '    0: (    MARK            Push markobject onto the stack.\n    1: N        NONE        Push None on the stack.\n    2: t        TUPLE      (MARK at 0) Build a tuple out of the topmost stack slice, after markobject.\n    3: .    STOP                       Stop the unpickling machine.\nhighest protocol among opcodes = 0\n', annotate=20)
check_dis(b'(((((((ttttttt.', '    0: (    MARK            Push markobject onto the stack.\n    1: (        MARK        Push markobject onto the stack.\n    2: (            MARK    Push markobject onto the stack.\n    3: (                MARK Push markobject onto the stack.\n    4: (                    MARK Push markobject onto the stack.\n    5: (                        MARK Push markobject onto the stack.\n    6: (                            MARK Push markobject onto the stack.\n    7: t                                TUPLE      (MARK at 6) Build a tuple out of the topmost stack slice, after markobject.\n    8: t                            TUPLE      (MARK at 5) Build a tuple out of the topmost stack slice, after markobject.\n    9: t                        TUPLE      (MARK at 4) Build a tuple out of the topmost stack slice, after markobject.\n   10: t                    TUPLE      (MARK at 3)     Build a tuple out of the topmost stack slice, after markobject.\n   11: t                TUPLE      (MARK at 2)         Build a tuple out of the topmost stack slice, after markobject.\n   12: t            TUPLE      (MARK at 1)             Build a tuple out of the topmost stack slice, after markobject.\n   13: t        TUPLE      (MARK at 0)                 Build a tuple out of the topmost stack slice, after markobject.\n   14: .    STOP                                       Stop the unpickling machine.\nhighest protocol among opcodes = 0\n', annotate=20)
print("DisTests::test_annotate: ok")
