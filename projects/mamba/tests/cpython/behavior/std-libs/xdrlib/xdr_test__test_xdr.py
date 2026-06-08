# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "behavior"
# case = "xdr_test__test_xdr"
# subject = "cpython.test_xdrlib.XDRTest.test_xdr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xdrlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xdrlib.py::XDRTest::test_xdr
"""Auto-ported test: XDRTest::test_xdr (CPython 3.12 oracle)."""


import unittest
from test.support import warnings_helper


xdrlib = warnings_helper.import_deprecated('xdrlib')


# --- test body ---
p = xdrlib.Packer()
s = b'hello world'
a = [b'what', b'is', b'hapnin', b'doctor']
p.pack_int(42)
p.pack_int(-17)
p.pack_uint(9)
p.pack_bool(True)
p.pack_bool(False)
p.pack_uhyper(45)
p.pack_float(1.9)
p.pack_double(1.9)
p.pack_string(s)
p.pack_list(range(5), p.pack_uint)
p.pack_array(a, p.pack_string)
data = p.get_buffer()
up = xdrlib.Unpacker(data)

assert up.get_position() == 0

assert up.unpack_int() == 42

assert up.unpack_int() == -17

assert up.unpack_uint() == 9

assert up.unpack_bool() is True
pos = up.get_position()

assert up.unpack_bool() is False
up.set_position(pos)

assert up.unpack_bool() is False

assert up.unpack_uhyper() == 45

assert abs(up.unpack_float() - 1.9) < 1e-07

assert abs(up.unpack_double() - 1.9) < 1e-07

assert up.unpack_string() == s

assert up.unpack_list(up.unpack_uint) == list(range(5))

assert up.unpack_array(up.unpack_string) == a
up.done()

try:
    up.unpack_uint()
    raise AssertionError('expected EOFError')
except EOFError:
    pass
print("XDRTest::test_xdr: ok")
