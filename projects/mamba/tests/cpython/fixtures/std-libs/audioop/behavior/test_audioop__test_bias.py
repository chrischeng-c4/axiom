# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "behavior"
# case = "test_audioop__test_bias"
# subject = "cpython.test_audioop.TestAudioop.test_bias"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audioop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audioop.py::TestAudioop::test_bias
"""Auto-ported test: TestAudioop::test_bias (CPython 3.12 oracle)."""


import sys
from test.support import warnings_helper
import unittest


audioop = warnings_helper.import_deprecated('audioop')

def pack(width, data):
    return b''.join((v.to_bytes(width, sys.byteorder, signed=True) for v in data))

def unpack(width, data):
    return [int.from_bytes(data[i:i + width], sys.byteorder, signed=True) for i in range(0, len(data), width)]

packs = {w: lambda *data, width=w: pack(width, data) for w in (1, 2, 3, 4)}

maxvalues = {w: (1 << 8 * w - 1) - 1 for w in (1, 2, 3, 4)}

minvalues = {w: -1 << 8 * w - 1 for w in (1, 2, 3, 4)}

datas = {1: b'\x00\x12E\xbb\x7f\x80\xff', 2: packs[2](0, 4660, 17767, -17767, 32767, -32768, -1), 3: packs[3](0, 1193046, 4548489, -4548489, 8388607, -8388608, -1), 4: packs[4](0, 305419896, 1164413355, -1164413355, 2147483647, -2147483648, -1)}

INVALID_DATA = [(b'abc', 0), (b'abc', 2), (b'ab', 3), (b'abc', 4)]


# --- test body ---
for w in (1, 2, 3, 4):
    for bias in (0, 1, -1, 127, -128, 2147483647, -2147483648):

        assert audioop.bias(b'', w, bias) == b''

        assert audioop.bias(bytearray(), w, bias) == b''

        assert audioop.bias(memoryview(b''), w, bias) == b''

assert audioop.bias(datas[1], 1, 1) == b'\x01\x13F\xbc\x80\x81\x00'

assert audioop.bias(datas[1], 1, -1) == b'\xff\x11D\xba~\x7f\xfe'

assert audioop.bias(datas[1], 1, 2147483647) == b'\xff\x11D\xba~\x7f\xfe'

assert audioop.bias(datas[1], 1, -2147483648) == datas[1]

assert audioop.bias(datas[2], 2, 1) == packs[2](1, 4661, 17768, -17766, -32768, -32767, 0)

assert audioop.bias(datas[2], 2, -1) == packs[2](-1, 4659, 17766, -17768, 32766, 32767, -2)

assert audioop.bias(datas[2], 2, 2147483647) == packs[2](-1, 4659, 17766, -17768, 32766, 32767, -2)

assert audioop.bias(datas[2], 2, -2147483648) == datas[2]

assert audioop.bias(datas[3], 3, 1) == packs[3](1, 1193047, 4548490, -4548488, -8388608, -8388607, 0)

assert audioop.bias(datas[3], 3, -1) == packs[3](-1, 1193045, 4548488, -4548490, 8388606, 8388607, -2)

assert audioop.bias(datas[3], 3, 2147483647) == packs[3](-1, 1193045, 4548488, -4548490, 8388606, 8388607, -2)

assert audioop.bias(datas[3], 3, -2147483648) == datas[3]

assert audioop.bias(datas[4], 4, 1) == packs[4](1, 305419897, 1164413356, -1164413354, -2147483648, -2147483647, 0)

assert audioop.bias(datas[4], 4, -1) == packs[4](-1, 305419895, 1164413354, -1164413356, 2147483646, 2147483647, -2)

assert audioop.bias(datas[4], 4, 2147483647) == packs[4](2147483647, -1842063753, -983070294, 983070292, -2, -1, 2147483646)

assert audioop.bias(datas[4], 4, -2147483648) == packs[4](-2147483648, -1842063752, -983070293, 983070293, -1, 0, 2147483647)
print("TestAudioop::test_bias: ok")
