# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "behavior"
# case = "genops_tests__test_genops"
# subject = "cpython.test_pickletools.GenopsTests.test_genops"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickletools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pickletools.py::GenopsTests::test_genops
"""Auto-ported test: GenopsTests::test_genops (CPython 3.12 oracle)."""


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
it = pickletools.genops(b'(I123\nK\x12J\x124Vxt.')

assert [(item[0].name,) + item[1:] for item in it] == [('MARK', None, 0), ('INT', 123, 1), ('BININT1', 18, 6), ('BININT', 2018915346, 8), ('TUPLE', None, 13), ('STOP', None, 14)]
print("GenopsTests::test_genops: ok")
