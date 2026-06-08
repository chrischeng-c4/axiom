# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "picklebuffer"
# dimension = "behavior"
# case = "pickle_buffer_test__test_raw_released"
# subject = "cpython.test_picklebuffer.PickleBufferTest.test_raw_released"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_picklebuffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_picklebuffer.py::PickleBufferTest::test_raw_released
"""Auto-ported test: PickleBufferTest::test_raw_released (CPython 3.12 oracle)."""


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
pb.release()
try:
    pb.raw()
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import types as _types_aR
    raises = _types_aR.SimpleNamespace(exception=_aR_e)
print("PickleBufferTest::test_raw_released: ok")
