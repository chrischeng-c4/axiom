# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bitfields"
# dimension = "behavior"
# case = "bit_field_test__test_anon_bitfields_uc52316e"
# subject = "cpython.test_bitfields.BitFieldTest.test_anon_bitfields"
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
    _fields_ = [('a', c_byte, 4), ('b', c_ubyte, 4)]

class Y(Structure):
    _anonymous_ = ['_']
    _fields_ = [('_', X)]

print("BitFieldTest::test_anon_bitfields: ok")
