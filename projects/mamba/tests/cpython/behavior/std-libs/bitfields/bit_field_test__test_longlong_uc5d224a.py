# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bitfields"
# dimension = "behavior"
# case = "bit_field_test__test_longlong_uc5d224a"
# subject = "cpython.test_bitfields.BitFieldTest.test_longlong"
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
    _fields_ = [('a', c_longlong, 1), ('b', c_longlong, 62), ('c', c_longlong, 1)]
assert sizeof(X) == sizeof(c_longlong)
x = X()
x.a, x.b, x.c = (-1, 7, -1)
assert (x.a, x.b, x.c) == (-1, 7, -1)

print("BitFieldTest::test_longlong: ok")
