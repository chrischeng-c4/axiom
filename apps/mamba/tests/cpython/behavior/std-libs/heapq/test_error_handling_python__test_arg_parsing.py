# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "test_error_handling_python__test_arg_parsing"
# subject = "cpython.test_heapq.TestErrorHandlingPython.test_arg_parsing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_heapq.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_heapq.py::TestErrorHandlingPython::test_arg_parsing
"""Auto-ported test: TestErrorHandlingPython::test_arg_parsing (CPython 3.12 oracle)."""


import random
import unittest
import doctest
from test.support import import_helper
from unittest import TestCase, skipUnless
from operator import itemgetter
from itertools import chain


'Unittests for heapq.'

py_heapq = import_helper.import_fresh_module('heapq', blocked=['_heapq'])

c_heapq = import_helper.import_fresh_module('heapq', fresh=['_heapq'])

func_names = ['heapify', 'heappop', 'heappush', 'heappushpop', 'heapreplace', '_heappop_max', '_heapreplace_max', '_heapify_max']

def load_tests(loader, tests, ignore):

    class HeapqMergeDocTestFinder:

        def find(self, *args, **kwargs):
            dtf = doctest.DocTestFinder()
            return dtf.find(py_heapq.merge)
    tests.addTests(doctest.DocTestSuite(py_heapq, test_finder=HeapqMergeDocTestFinder()))
    return tests

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __eq__(self, other):
        raise ZeroDivisionError
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class SideEffectLT:

    def __init__(self, value, heap):
        self.value = value
        self.heap = heap

    def __lt__(self, other):
        self.heap[:] = []
        return self.value < other.value


# --- test body ---
module = py_heapq
for f in (module.heapify, module.heappop, module.heappush, module.heapreplace, module.nlargest, module.nsmallest):

    try:
        f(10)
        raise AssertionError('expected (TypeError, AttributeError)')
    except (TypeError, AttributeError):
        pass
print("TestErrorHandlingPython::test_arg_parsing: ok")
