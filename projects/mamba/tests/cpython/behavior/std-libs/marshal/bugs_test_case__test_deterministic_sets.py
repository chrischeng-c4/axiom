# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "behavior"
# case = "bugs_test_case__test_deterministic_sets"
# subject = "cpython.test_marshal.BugsTestCase.test_deterministic_sets"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_marshal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_marshal.py::BugsTestCase::test_deterministic_sets
"""Auto-ported test: BugsTestCase::test_deterministic_sets (CPython 3.12 oracle)."""


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
for kind in ('set', 'frozenset'):
    for elements in ("float('nan'), b'a', b'b', b'c', 'x', 'y', 'z'", "('Spam', 0), ('Spam', 1), ('Spam', 2), ('Spam', 3), ('Spam', 4), ('Spam', 5)"):
        s = f'{kind}([{elements}])'
        if sys.hash_info.algorithm in {'fnv', 'siphash24'}:
            args = ['-c', f'print({s})']
            _, repr_0, _ = assert_python_ok(*args, PYTHONHASHSEED='0')
            _, repr_1, _ = assert_python_ok(*args, PYTHONHASHSEED='1')

            assert repr_0 != repr_1
        args = ['-c', f'import marshal; print(marshal.dumps({s}))']
        _, dump_0, _ = assert_python_ok(*args, PYTHONHASHSEED='0')
        _, dump_1, _ = assert_python_ok(*args, PYTHONHASHSEED='1')

        assert dump_0 == dump_1
print("BugsTestCase::test_deterministic_sets: ok")
