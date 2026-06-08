# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "behavior"
# case = "test_audioop__test_string"
# subject = "cpython.test_audioop.TestAudioop.test_string"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audioop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audioop.py::TestAudioop::test_string
"""Auto-ported test: TestAudioop::test_string (CPython 3.12 oracle)."""


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
data = 'abcd'
size = 2

try:
    audioop.getsample(data, size, 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.max(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.minmax(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.avg(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.rms(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.avgpp(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.maxpp(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.cross(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.mul(data, size, 1.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.tomono(data, size, 0.5, 0.5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.tostereo(data, size, 0.5, 0.5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.add(data, data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.bias(data, size, 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.reverse(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.lin2lin(data, size, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.ratecv(data, size, 1, 1, 1, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.lin2ulaw(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.lin2alaw(data, size)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    audioop.lin2adpcm(data, size, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestAudioop::test_string: ok")
