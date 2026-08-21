# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frombuffer"
# dimension = "behavior"
# case = "test__test_abstract_ucbb930e"
# subject = "cpython.test_frombuffer.Test.test_abstract"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_frombuffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import array
import gc
from ctypes import _Pointer, _SimpleCData, _CFuncPtr
try:
    Array.from_buffer(bytearray(10))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    Structure.from_buffer(bytearray(10))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    Union.from_buffer(bytearray(10))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    _CFuncPtr.from_buffer(bytearray(10))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    _Pointer.from_buffer(bytearray(10))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    _SimpleCData.from_buffer(bytearray(10))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    Array.from_buffer_copy(b'123')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    Structure.from_buffer_copy(b'123')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    Union.from_buffer_copy(b'123')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    _CFuncPtr.from_buffer_copy(b'123')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    _Pointer.from_buffer_copy(b'123')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    _SimpleCData.from_buffer_copy(b'123')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("Test::test_abstract: ok")
