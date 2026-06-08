# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "behavior"
# case = "test_audioop__test_lin2adpcm"
# subject = "cpython.test_audioop.TestAudioop.test_lin2adpcm"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audioop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audioop.py::TestAudioop::test_lin2adpcm
"""Auto-ported test: TestAudioop::test_lin2adpcm (CPython 3.12 oracle)."""


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

assert audioop.lin2adpcm(datas[1], 1, None) == (b'\x07\x7f\x7f', (-221, 39))

assert audioop.lin2adpcm(bytearray(datas[1]), 1, None) == (b'\x07\x7f\x7f', (-221, 39))

assert audioop.lin2adpcm(memoryview(datas[1]), 1, None) == (b'\x07\x7f\x7f', (-221, 39))
for w in (2, 3, 4):

    assert audioop.lin2adpcm(datas[w], w, None) == (b'\x07\x7f\x7f', (31, 39))
for w in (1, 2, 3, 4):

    assert audioop.lin2adpcm(b'\x00' * w * 10, w, None) == (b'\x00' * 5, (0, 0))
print("TestAudioop::test_lin2adpcm: ok")
