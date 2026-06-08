# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "behavior"
# case = "optimized_pickle_tests__test_optimize_long_binget"
# subject = "cpython.test_pickletools.OptimizedPickleTests.test_optimize_long_binget"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickletools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pickletools.py::OptimizedPickleTests::test_optimize_long_binget
"""Auto-ported test: OptimizedPickleTests::test_optimize_long_binget (CPython 3.12 oracle)."""


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
test_pickle_to_2x = None
test_framed_write_sizes_with_delayed_writer = None
data = [str(i) for i in range(257)]
data.append(data[-1])
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    pickled = pickle.dumps(data, proto)
    unpickled = pickle.loads(pickled)

    assert unpickled == data

    assert unpickled[-1] is unpickled[-2]
    pickled2 = pickletools.optimize(pickled)
    unpickled2 = pickle.loads(pickled2)

    assert unpickled2 == data

    assert unpickled2[-1] is unpickled2[-2]

    assert pickle.LONG_BINGET not in pickled2

    assert pickle.LONG_BINPUT not in pickled2
print("OptimizedPickleTests::test_optimize_long_binget: ok")
