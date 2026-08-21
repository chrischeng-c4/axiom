# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "picklebuffer"
# dimension = "behavior"
# case = "pickle_buffer_test__test_cycle"
# subject = "cpython.test_picklebuffer.PickleBufferTest.test_cycle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_picklebuffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_picklebuffer.py::PickleBufferTest::test_cycle
"""Auto-ported test: PickleBufferTest::test_cycle (CPython 3.12 oracle)."""


import gc
from pickle import PickleBuffer
import weakref
import unittest
from test.support import import_helper


'Unit tests for the PickleBuffer object.\n\nPickling tests themselves are in pickletester.py.\n'

class B(bytes):
    pass


# --- test body ---
b = B(b'foo')
pb = PickleBuffer(b)
b.cycle = pb
wpb = weakref.ref(pb)
del b, pb
gc.collect()

assert wpb() is None
print("PickleBufferTest::test_cycle: ok")
