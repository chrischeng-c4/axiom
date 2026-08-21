# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "picklebuffer"
# dimension = "behavior"
# case = "pickle_buffer_test__test_constructor_failure"
# subject = "cpython.test_picklebuffer.PickleBufferTest.test_constructor_failure"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_picklebuffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_picklebuffer.py::PickleBufferTest::test_constructor_failure
"""Auto-ported test: PickleBufferTest::test_constructor_failure (CPython 3.12 oracle)."""


import gc
from pickle import PickleBuffer
import weakref
import unittest
from test.support import import_helper


'Unit tests for the PickleBuffer object.\n\nPickling tests themselves are in pickletester.py.\n'

class B(bytes):
    pass


# --- test body ---
try:
    PickleBuffer()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    PickleBuffer('foo')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
m = memoryview(b'foo')
m.release()
try:
    PickleBuffer(m)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PickleBufferTest::test_constructor_failure: ok")
