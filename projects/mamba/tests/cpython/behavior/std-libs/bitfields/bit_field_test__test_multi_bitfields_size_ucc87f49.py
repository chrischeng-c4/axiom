# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bitfields"
# dimension = "behavior"
# case = "bit_field_test__test_multi_bitfields_size_ucc87f49"
# subject = "cpython.test_bitfields.BitFieldTest.test_multi_bitfields_size"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_bitfields.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import os
import _ctypes_test

def fail_fields(*fields):
    return get_except(type(Structure), 'X', (), {'_fields_': fields})

def get_except(func, *args, **kw):
    try:
        func(*args, **kw)
    except Exception as detail:
        return (detail.__class__, str(detail))

class X(Structure):
    _fields_ = [('a', c_short, 1), ('b', c_short, 14), ('c', c_short, 1)]
assert sizeof(X) == sizeof(c_short)

class X(Structure):
    _fields_ = [('a', c_short, 1), ('a1', c_short), ('b', c_short, 14), ('c', c_short, 1)]
assert sizeof(X) == sizeof(c_short) * 3
assert X.a.offset == 0
assert X.a1.offset == sizeof(c_short)
assert X.b.offset == sizeof(c_short) * 2
assert X.c.offset == sizeof(c_short) * 2

class X(Structure):
    _fields_ = [('a', c_short, 3), ('b', c_short, 14), ('c', c_short, 14)]
assert sizeof(X) == sizeof(c_short) * 3
assert X.a.offset == sizeof(c_short) * 0
assert X.b.offset == sizeof(c_short) * 1
assert X.c.offset == sizeof(c_short) * 2

print("BitFieldTest::test_multi_bitfields_size: ok")
