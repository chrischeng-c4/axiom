# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_move"
# subject = "cpython.test_mmap.MmapTests.test_move"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_move
"""Auto-ported test: MmapTests::test_move (CPython 3.12 oracle)."""


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
with open(TESTFN, 'wb+') as f:
    f.write(b'ABCDEabcde')
    f.flush()
    mf = mmap.mmap(f.fileno(), 10)
    mf.move(5, 0, 5)

    assert mf[:] == b'ABCDEABCDE'
    mf.close()
data = b'0123456789'
for dest in range(len(data)):
    for src in range(len(data)):
        for count in range(len(data) - max(dest, src)):
            expected = data[:dest] + data[src:src + count] + data[dest + count:]
            m = mmap.mmap(-1, len(data))
            m[:] = data
            m.move(dest, src, count)

            assert m[:] == expected
            m.close()
m = mmap.mmap(-1, 100)
offsets = [-100, -1, 0, 1, 100]
for source, dest, size in itertools.product(offsets, offsets, offsets):
    try:
        m.move(source, dest, size)
    except ValueError:
        pass
offsets = [(-1, -1, -1), (-1, -1, 0), (-1, 0, -1), (0, -1, -1), (-1, 0, 0), (0, -1, 0), (0, 0, -1)]
for source, dest, size in offsets:

    try:
        m.move(source, dest, size)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
m.close()
m = mmap.mmap(-1, 1)

try:
    m.move(0, 0, 2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    m.move(1, 0, 1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    m.move(0, 1, 1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
m.move(0, 0, 1)
m.move(0, 0, 0)
print("MmapTests::test_move: ok")
