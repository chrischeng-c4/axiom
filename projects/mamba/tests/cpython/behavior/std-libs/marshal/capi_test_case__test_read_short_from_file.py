# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "behavior"
# case = "capi_test_case__test_read_short_from_file"
# subject = "cpython.test_marshal.CAPI_TestCase.test_read_short_from_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_marshal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_marshal.py::CAPI_TestCase::test_read_short_from_file
"""Auto-ported test: CAPI_TestCase::test_read_short_from_file (CPython 3.12 oracle)."""


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

class InstancingTestCase(unittest.TestCase, HelperMixin):
    keys = (123, 1.2345, 'abc', (123, 'abc'), frozenset({123, 'abc'}))

    def helper3(self, rsample, recursive=False, simple=False):
        sample = (rsample, rsample)
        n0 = CollectObjectIDs(set(), sample)
        for v in range(3, marshal.version + 1):
            s3 = marshal.dumps(sample, v)
            n3 = CollectObjectIDs(set(), marshal.loads(s3))
            self.assertEqual(n3, n0)
        if not recursive:
            s2 = marshal.dumps(sample, 2)
            n2 = CollectObjectIDs(set(), marshal.loads(s2))
            self.assertGreater(n2, n0)
            if not simple:
                self.assertGreater(len(s2), len(s3))
            else:
                self.assertGreaterEqual(len(s2), len(s3))

    def testInt(self):
        intobj = 123321
        self.helper(intobj)
        self.helper3(intobj, simple=True)

    def testFloat(self):
        floatobj = 1.2345
        self.helper(floatobj)
        self.helper3(floatobj)

    def testStr(self):
        strobj = 'abcde' * 3
        self.helper(strobj)
        self.helper3(strobj)

    def testBytes(self):
        bytesobj = b'abcde' * 3
        self.helper(bytesobj)
        self.helper3(bytesobj)

    def testList(self):
        for obj in self.keys:
            listobj = [obj, obj]
            self.helper(listobj)
            self.helper3(listobj)

    def testTuple(self):
        for obj in self.keys:
            tupleobj = (obj, obj)
            self.helper(tupleobj)
            self.helper3(tupleobj)

    def testSet(self):
        for obj in self.keys:
            setobj = {(obj, 1), (obj, 2)}
            self.helper(setobj)
            self.helper3(setobj)

    def testFrozenSet(self):
        for obj in self.keys:
            frozensetobj = frozenset({(obj, 1), (obj, 2)})
            self.helper(frozensetobj)
            self.helper3(frozensetobj)

    def testDict(self):
        for obj in self.keys:
            dictobj = {'hello': obj, 'goodbye': obj, obj: 'hello'}
            self.helper(dictobj)
            self.helper3(dictobj)

    def testModule(self):
        with open(__file__, 'rb') as f:
            code = f.read()
        if __file__.endswith('.py'):
            code = compile(code, __file__, 'exec')
        self.helper(code)
        self.helper3(code)

    def testRecursion(self):
        obj = 1.2345
        d = {'hello': obj, 'goodbye': obj, obj: 'hello'}
        d['self'] = d
        self.helper3(d, recursive=True)
        l = [obj, obj]
        l.append(l)
        self.helper3(l, recursive=True)

class CompatibilityTestCase(unittest.TestCase):

    def _test(self, version):
        with open(__file__, 'rb') as f:
            code = f.read()
        if __file__.endswith('.py'):
            code = compile(code, __file__, 'exec')
        data = marshal.dumps(code, version)
        marshal.loads(data)

    def test0To3(self):
        self._test(0)

    def test1To3(self):
        self._test(1)

    def test2To3(self):
        self._test(2)

    def test3To3(self):
        self._test(3)

class InterningTestCase(unittest.TestCase, HelperMixin):
    strobj = 'this is an interned string'
    strobj = sys.intern(strobj)

    def testIntern(self):
        s = marshal.loads(marshal.dumps(self.strobj))
        self.assertEqual(s, self.strobj)
        self.assertEqual(id(s), id(self.strobj))
        s2 = sys.intern(s)
        self.assertEqual(id(s2), id(s))

    def testNoIntern(self):
        s = marshal.loads(marshal.dumps(self.strobj, 2))
        self.assertEqual(s, self.strobj)
        self.assertNotEqual(id(s), id(self.strobj))
        s2 = sys.intern(s)
        self.assertNotEqual(id(s2), id(s))


# --- test body ---
with open(os_helper.TESTFN, 'wb') as f:
    f.write(b'4\x12xxxx')
r, p = _testcapi.pymarshal_read_short_from_file(os_helper.TESTFN)
os_helper.unlink(os_helper.TESTFN)

assert r == 4660

assert p == 2
with open(os_helper.TESTFN, 'wb') as f:
    f.write(b'\x12')
try:
    _testcapi.pymarshal_read_short_from_file(os_helper.TESTFN)
    raise AssertionError('expected EOFError')
except EOFError:
    pass
os_helper.unlink(os_helper.TESTFN)
print("CAPI_TestCase::test_read_short_from_file: ok")
