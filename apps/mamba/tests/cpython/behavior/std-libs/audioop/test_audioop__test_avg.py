# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "behavior"
# case = "test_audioop__test_avg"
# subject = "cpython.test_audioop.TestAudioop.test_avg"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audioop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audioop.py::TestAudioop::test_avg
"""Auto-ported test: TestAudioop::test_avg (CPython 3.12 oracle)."""


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

    assert audioop.avg(b'', w) == 0

    assert audioop.avg(bytearray(), w) == 0

    assert audioop.avg(memoryview(b''), w) == 0
    p = packs[w]

    assert audioop.avg(p(5), w) == 5

    assert audioop.avg(p(5, 8), w) == 6

    assert audioop.avg(p(5, -8), w) == -2

    assert audioop.avg(p(maxvalues[w], maxvalues[w]), w) == maxvalues[w]

    assert audioop.avg(p(minvalues[w], minvalues[w]), w) == minvalues[w]

assert audioop.avg(packs[4](1342177280, 1879048192), 4) == 1610612736

assert audioop.avg(packs[4](-1342177280, -1879048192), 4) == -1610612736
print("TestAudioop::test_avg: ok")
