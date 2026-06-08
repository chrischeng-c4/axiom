# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "behavior"
# case = "test_audioop__test_ratecv"
# subject = "cpython.test_audioop.TestAudioop.test_ratecv"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audioop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audioop.py::TestAudioop::test_ratecv
"""Auto-ported test: TestAudioop::test_ratecv (CPython 3.12 oracle)."""


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

    assert audioop.ratecv(b'', w, 1, 8000, 8000, None) == (b'', (-1, ((0, 0),)))

    assert audioop.ratecv(bytearray(), w, 1, 8000, 8000, None) == (b'', (-1, ((0, 0),)))

    assert audioop.ratecv(memoryview(b''), w, 1, 8000, 8000, None) == (b'', (-1, ((0, 0),)))

    assert audioop.ratecv(b'', w, 5, 8000, 8000, None) == (b'', (-1, ((0, 0),) * 5))

    assert audioop.ratecv(b'', w, 1, 8000, 16000, None) == (b'', (-2, ((0, 0),)))

    assert audioop.ratecv(datas[w], w, 1, 8000, 8000, None)[0] == datas[w]

    assert audioop.ratecv(datas[w], w, 1, 8000, 8000, None, 1, 0)[0] == datas[w]
state = None
d1, state = audioop.ratecv(b'\x00\x01\x02', 1, 1, 8000, 16000, state)
d2, state = audioop.ratecv(b'\x00\x01\x02', 1, 1, 8000, 16000, state)

assert d1 + d2 == b'\x00\x00\x01\x01\x02\x01\x00\x00\x01\x01\x02'
for w in (1, 2, 3, 4):
    d0, state0 = audioop.ratecv(datas[w], w, 1, 8000, 16000, None)
    d, state = (b'', None)
    for i in range(0, len(datas[w]), w):
        d1, state = audioop.ratecv(datas[w][i:i + w], w, 1, 8000, 16000, state)
        d += d1

    assert d == d0

    assert state == state0
expected = {1: packs[1](0, 13, 55, -38, 85, -75, -20), 2: packs[2](0, 3495, 14199, -9776, 22131, -19044, -4762), 3: packs[3](0, 894784, 3635062, -2502602, 5665804, -4875005, -1218752), 4: packs[4](0, 229064922, 930576246, -640665954, 1450446246, -1248001174, -312000294)}
for w in (1, 2, 3, 4):

    assert audioop.ratecv(datas[w], w, 1, 8000, 8000, None, 3, 1)[0] == expected[w]

    assert audioop.ratecv(datas[w], w, 1, 8000, 8000, None, 30, 10)[0] == expected[w]

try:
    audioop.ratecv(b'', 1, 1, 8000, 8000, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.ratecv(b'', 1, 1, 8000, 8000, (1, (42,)))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestAudioop::test_ratecv: ok")
