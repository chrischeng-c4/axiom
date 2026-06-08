# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "picklebuffer"
# dimension = "behavior"
# case = "pickle_buffer_test__test_basics"
# subject = "cpython.test_picklebuffer.PickleBufferTest.test_basics"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_picklebuffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_picklebuffer.py::PickleBufferTest::test_basics
"""Auto-ported test: PickleBufferTest::test_basics (CPython 3.12 oracle)."""


import gc
from pickle import PickleBuffer
import weakref
import unittest
from test.support import import_helper


'Unit tests for the PickleBuffer object.\n\nPickling tests themselves are in pickletester.py.\n'

class B(bytes):
    pass


# --- test body ---
pb = PickleBuffer(b'foo')

assert b'foo' == bytes(pb)
with memoryview(pb) as m:

    assert m.readonly
pb = PickleBuffer(bytearray(b'foo'))

assert b'foo' == bytes(pb)
with memoryview(pb) as m:

    assert not m.readonly
    m[0] = 48

assert b'0oo' == bytes(pb)
print("PickleBufferTest::test_basics: ok")
