# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "behavior"
# case = "test_audioop__test_adpcm2lin"
# subject = "cpython.test_audioop.TestAudioop.test_adpcm2lin"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audioop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audioop.py::TestAudioop::test_adpcm2lin
"""Auto-ported test: TestAudioop::test_adpcm2lin (CPython 3.12 oracle)."""


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

assert audioop.adpcm2lin(b'\x07\x7f\x7f', 1, None) == (b'\x00\x00\x00\xff\x00\xff', (-179, 40))

assert audioop.adpcm2lin(bytearray(b'\x07\x7f\x7f'), 1, None) == (b'\x00\x00\x00\xff\x00\xff', (-179, 40))

assert audioop.adpcm2lin(memoryview(b'\x07\x7f\x7f'), 1, None) == (b'\x00\x00\x00\xff\x00\xff', (-179, 40))

assert audioop.adpcm2lin(b'\x07\x7f\x7f', 2, None) == (packs[2](0, 11, 41, -22, 114, -179), (-179, 40))

assert audioop.adpcm2lin(b'\x07\x7f\x7f', 3, None) == (packs[3](0, 2816, 10496, -5632, 29184, -45824), (-179, 40))

assert audioop.adpcm2lin(b'\x07\x7f\x7f', 4, None) == (packs[4](0, 720896, 2686976, -1441792, 7471104, -11730944), (-179, 40))
for w in (1, 2, 3, 4):

    assert audioop.adpcm2lin(b'\x00' * 5, w, None) == (b'\x00' * w * 10, (0, 0))
print("TestAudioop::test_adpcm2lin: ok")
