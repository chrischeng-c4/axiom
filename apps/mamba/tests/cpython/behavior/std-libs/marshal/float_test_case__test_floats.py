# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "behavior"
# case = "float_test_case__test_floats"
# subject = "cpython.test_marshal.FloatTestCase.test_floats"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_marshal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_marshal.py::FloatTestCase::test_floats
"""Auto-ported test: FloatTestCase::test_floats (CPython 3.12 oracle)."""


from test import support
from test.support import os_helper, requires_debug_ranges
from test.support.script_helper import assert_python_ok
import array
import io
import marshal
import sys
import unittest
import os
import types
import textwrap


try:
    import _testcapi
except ImportError:
    _testcapi = None

class HelperMixin:

    def helper(self, sample, *extra):
        new = marshal.loads(marshal.dumps(sample, *extra))
        self.assertEqual(sample, new)
        try:
            with open(os_helper.TESTFN, 'wb') as f:
                marshal.dump(sample, f, *extra)
            with open(os_helper.TESTFN, 'rb') as f:
                new = marshal.load(f)
            self.assertEqual(sample, new)
        finally:
            os_helper.unlink(os_helper.TESTFN)

LARGE_SIZE = 2 ** 31

pointer_size = 8 if sys.maxsize > 4294967295 else 4

class NullWriter:

    def write(self, s):
        pass

def CollectObjectIDs(ids, obj):
    """Collect object ids seen in a structure"""
    if id(obj) in ids:
        return
    ids.add(id(obj))
    if isinstance(obj, (list, tuple, set, frozenset)):
        for e in obj:
            CollectObjectIDs(ids, e)
    elif isinstance(obj, dict):
        for k, v in obj.items():
            CollectObjectIDs(ids, k)
            CollectObjectIDs(ids, v)
    return len(ids)


# --- test body ---
def helper(sample, *extra):
    new = marshal.loads(marshal.dumps(sample, *extra))

    assert sample == new
    try:
        with open(os_helper.TESTFN, 'wb') as f:
            marshal.dump(sample, f, *extra)
        with open(os_helper.TESTFN, 'rb') as f:
            new = marshal.load(f)

        assert sample == new
    finally:
        os_helper.unlink(os_helper.TESTFN)
small = 1e-25
n = sys.maxsize * 3.7e+250
while n > small:
    for expected in (-n, n):
        helper(float(expected))
    n /= 123.4567
f = 0.0
s = marshal.dumps(f, 2)
got = marshal.loads(s)

assert f == got
s = marshal.dumps(f, 1)
got = marshal.loads(s)

assert f == got
n = sys.maxsize * 3.7e-250
while n < small:
    for expected in (-n, n):
        f = float(expected)
        helper(f)
        helper(f, 1)
    n *= 123.4567
print("FloatTestCase::test_floats: ok")
