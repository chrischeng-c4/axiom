# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "behavior"
# case = "test_audioop__test_ulaw2lin"
# subject = "cpython.test_audioop.TestAudioop.test_ulaw2lin"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audioop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audioop.py::TestAudioop::test_ulaw2lin
"""Auto-ported test: TestAudioop::test_ulaw2lin (CPython 3.12 oracle)."""


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
encoded = b'\x00\x0e(?Wjv|~\x7f\x80\x8e\xa8\xbf\xd7\xea\xf6\xfc\xfe\xff'
src = [-8031, -4447, -1471, -495, -163, -53, -18, -6, -2, 0, 8031, 4447, 1471, 495, 163, 53, 18, 6, 2, 0]
for w in (1, 2, 3, 4):
    decoded = packs[w](*(x << w * 8 >> 14 for x in src))

    assert audioop.ulaw2lin(encoded, w) == decoded

    assert audioop.ulaw2lin(bytearray(encoded), w) == decoded

    assert audioop.ulaw2lin(memoryview(encoded), w) == decoded
encoded = bytes(range(127)) + bytes(range(128, 256))
for w in (2, 3, 4):
    decoded = audioop.ulaw2lin(encoded, w)

    assert audioop.lin2ulaw(decoded, w) == encoded
print("TestAudioop::test_ulaw2lin: ok")
