# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "picklebuffer"
# dimension = "behavior"
# case = "pickle_buffer_test__test_ndarray_2d"
# subject = "cpython.test_picklebuffer.PickleBufferTest.test_ndarray_2d"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_picklebuffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_picklebuffer.py::PickleBufferTest::test_ndarray_2d
"""Auto-ported test: PickleBufferTest::test_ndarray_2d (CPython 3.12 oracle)."""


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
arr = ndarray(list(range(12)), shape=(4, 3), format='<i')

assert arr.c_contiguous

assert not arr.f_contiguous
pb = PickleBuffer(arr)
check_memoryview(pb, arr)
arr = arr[::2]

assert not arr.c_contiguous

assert not arr.f_contiguous
pb = PickleBuffer(arr)
check_memoryview(pb, arr)
arr = ndarray(list(range(12)), shape=(3, 4), strides=(4, 12), format='<i')

assert arr.f_contiguous

assert not arr.c_contiguous
pb = PickleBuffer(arr)
check_memoryview(pb, arr)
print("PickleBufferTest::test_ndarray_2d: ok")
