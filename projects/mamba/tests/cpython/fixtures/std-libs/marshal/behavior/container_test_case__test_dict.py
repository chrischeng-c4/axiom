# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "behavior"
# case = "container_test_case__test_dict"
# subject = "cpython.test_marshal.ContainerTestCase.test_dict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_marshal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_marshal.py::ContainerTestCase::test_dict
"""Auto-ported test: ContainerTestCase::test_dict (CPython 3.12 oracle)."""


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
d = {'astring': 'foo@bar.baz.spam', 'afloat': 7283.43, 'anint': 2 ** 20, 'ashortlong': 2, 'alist': ['.zyx.41'], 'atuple': ('.zyx.41',) * 10, 'aboolean': False, 'aunicode': 'Andrè Previn'}

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
helper(d)
print("ContainerTestCase::test_dict: ok")
