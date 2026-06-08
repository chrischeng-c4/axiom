# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "picklebuffer"
# dimension = "behavior"
# case = "pickle_buffer_test__test_raw_non_contiguous"
# subject = "cpython.test_picklebuffer.PickleBufferTest.test_raw_non_contiguous"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_picklebuffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_picklebuffer.py::PickleBufferTest::test_raw_non_contiguous
"""Auto-ported test: PickleBufferTest::test_raw_non_contiguous (CPython 3.12 oracle)."""


import gc
from pickle import PickleBuffer
import weakref
import unittest
from test.support import import_helper


'Unit tests for the PickleBuffer object.\n\nPickling tests themselves are in pickletester.py.\n'

class B(bytes):
    pass


# --- test body ---
def check_memoryview(pb, equiv):
    with memoryview(pb) as m:
        with memoryview(equiv) as expected:

            assert m.nbytes == expected.nbytes

            assert m.readonly == expected.readonly

            assert m.itemsize == expected.itemsize

            assert m.shape == expected.shape

            assert m.strides == expected.strides

            assert m.c_contiguous == expected.c_contiguous

            assert m.f_contiguous == expected.f_contiguous

            assert m.format == expected.format

            assert m.tobytes() == expected.tobytes()

def check_raw(obj, equiv):
    pb = PickleBuffer(obj)
    with pb.raw() as m:

        assert isinstance(m, memoryview)
        check_memoryview(m, equiv)

def check_raw_non_contiguous(obj):
    pb = PickleBuffer(obj)
    try:
        pb.raw()
        raise AssertionError('expected BufferError')
    except BufferError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('non-contiguous', str(_aR_e))
ndarray = import_helper.import_module('_testbuffer').ndarray
arr = ndarray(list(range(6)), shape=(6,), format='<i')[::2]
check_raw_non_contiguous(arr)
arr = ndarray(list(range(12)), shape=(4, 3), format='<i')[::2]
check_raw_non_contiguous(arr)
print("PickleBufferTest::test_raw_non_contiguous: ok")
