# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "behavior"
# case = "test_audioop__test_lin2lin"
# subject = "cpython.test_audioop.TestAudioop.test_lin2lin"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audioop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audioop.py::TestAudioop::test_lin2lin
"""Auto-ported test: TestAudioop::test_lin2lin (CPython 3.12 oracle)."""


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

    assert audioop.lin2lin(datas[w], w, w) == datas[w]

    assert audioop.lin2lin(bytearray(datas[w]), w, w) == datas[w]

    assert audioop.lin2lin(memoryview(datas[w]), w, w) == datas[w]

assert audioop.lin2lin(datas[1], 1, 2) == packs[2](0, 4608, 17664, -17664, 32512, -32768, -256)

assert audioop.lin2lin(datas[1], 1, 3) == packs[3](0, 1179648, 4521984, -4521984, 8323072, -8388608, -65536)

assert audioop.lin2lin(datas[1], 1, 4) == packs[4](0, 301989888, 1157627904, -1157627904, 2130706432, -2147483648, -16777216)

assert audioop.lin2lin(datas[2], 2, 1) == b'\x00\x12E\xba\x7f\x80\xff'

assert audioop.lin2lin(datas[2], 2, 3) == packs[3](0, 1192960, 4548352, -4548352, 8388352, -8388608, -256)

assert audioop.lin2lin(datas[2], 2, 4) == packs[4](0, 305397760, 1164378112, -1164378112, 2147418112, -2147483648, -65536)

assert audioop.lin2lin(datas[3], 3, 1) == b'\x00\x12E\xba\x7f\x80\xff'

assert audioop.lin2lin(datas[3], 3, 2) == packs[2](0, 4660, 17767, -17768, 32767, -32768, -1)

assert audioop.lin2lin(datas[3], 3, 4) == packs[4](0, 305419776, 1164413184, -1164413184, 2147483392, -2147483648, -256)

assert audioop.lin2lin(datas[4], 4, 1) == b'\x00\x12E\xba\x7f\x80\xff'

assert audioop.lin2lin(datas[4], 4, 2) == packs[2](0, 4660, 17767, -17768, 32767, -32768, -1)

assert audioop.lin2lin(datas[4], 4, 3) == packs[3](0, 1193046, 4548489, -4548490, 8388607, -8388608, -1)
print("TestAudioop::test_lin2lin: ok")
