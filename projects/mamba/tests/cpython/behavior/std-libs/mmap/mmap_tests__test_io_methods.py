# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_io_methods"
# subject = "cpython.test_mmap.MmapTests.test_io_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_io_methods
"""Auto-ported test: MmapTests::test_io_methods (CPython 3.12 oracle)."""


from test.support import requires, _2G, _4G, gc_collect, cpython_only, is_emscripten
from test.support.import_helper import import_module
from test.support.os_helper import TESTFN, unlink
import unittest
import os
import re
import itertools
import random
import socket
import string
import sys
import weakref


mmap = import_module('mmap')

PAGESIZE = mmap.PAGESIZE

tagname_prefix = f'python_{os.getpid()}_test_mmap'

def random_tagname(length=10):
    suffix = ''.join(random.choices(string.ascii_uppercase, k=length))
    return f'{tagname_prefix}_{suffix}'

if is_emscripten:
    raise unittest.SkipTest("incompatible with Emscripten's mmap emulation.")


# --- test body ---
if os.path.exists(TESTFN):
    os.unlink(TESTFN)
data = b'0123456789'
with open(TESTFN, 'wb') as fp:
    fp.write(b'x' * len(data))
with open(TESTFN, 'r+b') as f:
    m = mmap.mmap(f.fileno(), len(data))
for i in range(len(data)):

    assert m.tell() == i
    m.write_byte(data[i])

    assert m.tell() == i + 1

try:
    m.write_byte(b'x'[0])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert m[:] == data
m.seek(0)
for i in range(len(data)):

    assert m.tell() == i

    assert m.read_byte() == data[i]

    assert m.tell() == i + 1

try:
    m.read_byte()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
m.seek(3)

assert m.read(3) == b'345'

assert m.tell() == 6
m.seek(3)
m.write(b'bar')

assert m.tell() == 6

assert m[:] == b'012bar6789'
m.write(bytearray(b'baz'))

assert m.tell() == 9

assert m[:] == b'012barbaz9'

try:
    m.write(b'ba')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("MmapTests::test_io_methods: ok")
