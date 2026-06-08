# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "behavior"
# case = "instancing_test_case__test_list"
# subject = "cpython.test_marshal.InstancingTestCase.testList"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_marshal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_marshal.py::InstancingTestCase::testList
"""Auto-ported test: InstancingTestCase::testList (CPython 3.12 oracle)."""


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
keys = (123, 1.2345, 'abc', (123, 'abc'), frozenset({123, 'abc'}))

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

def helper3(rsample, recursive=False, simple=False):
    sample = (rsample, rsample)
    n0 = CollectObjectIDs(set(), sample)
    for v in range(3, marshal.version + 1):
        s3 = marshal.dumps(sample, v)
        n3 = CollectObjectIDs(set(), marshal.loads(s3))

        assert n3 == n0
    if not recursive:
        s2 = marshal.dumps(sample, 2)
        n2 = CollectObjectIDs(set(), marshal.loads(s2))

        assert n2 > n0
        if not simple:

            assert len(s2) > len(s3)
        else:

            assert len(s2) >= len(s3)
for obj in keys:
    listobj = [obj, obj]
    helper(listobj)
    helper3(listobj)
print("InstancingTestCase::testList: ok")
