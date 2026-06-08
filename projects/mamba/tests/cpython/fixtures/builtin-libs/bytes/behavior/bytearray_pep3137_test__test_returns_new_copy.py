# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytearray_pep3137_test__test_returns_new_copy"
# subject = "cpython.test_bytes.BytearrayPEP3137Test.test_returns_new_copy"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bytes.py::BytearrayPEP3137Test::test_returns_new_copy
"""Auto-ported test: BytearrayPEP3137Test::test_returns_new_copy (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
def marshal(x):
    return bytearray(x)
val = marshal(b'1234')
for methname in ('zfill', 'rjust', 'ljust', 'center'):
    method = getattr(val, methname)
    newval = method(3)

    assert val == newval

    assert val is not newval
for expr in ('val.split()[0]', 'val.rsplit()[0]', 'val.partition(b".")[0]', 'val.rpartition(b".")[2]', 'val.splitlines()[0]', 'val.replace(b"", b"")'):
    newval = eval(expr)

    assert val == newval

    assert val is not newval
sep = marshal(b'')
newval = sep.join([val])

assert val == newval

assert val is not newval
print("BytearrayPEP3137Test::test_returns_new_copy: ok")
