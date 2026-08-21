# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_offset"
# subject = "cpython.test_mmap.MmapTests.test_offset"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_offset
"""Auto-ported test: MmapTests::test_offset (CPython 3.12 oracle)."""


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
def make_mmap_file(f, halfsize):
    f.write(b'\x00' * halfsize)
    f.write(b'foo')
    f.write(b'\x00' * (halfsize - 3))
    f.flush()
    return mmap.mmap(f.fileno(), 0)
if os.path.exists(TESTFN):
    os.unlink(TESTFN)
f = open(TESTFN, 'w+b')
try:
    halfsize = mmap.ALLOCATIONGRANULARITY
    m = make_mmap_file(f, halfsize)
    m.close()
    f.close()
    mapsize = halfsize * 2
    f = open(TESTFN, 'r+b')
    for offset in [-2, -1, None]:
        try:
            m = mmap.mmap(f.fileno(), mapsize, offset=offset)

            assert 0 == 1
        except (ValueError, TypeError, OverflowError):
            pass
        else:

            assert 0 == 0
    f.close()
    f = open(TESTFN, 'r+b')
    m = mmap.mmap(f.fileno(), mapsize - halfsize, offset=halfsize)

    assert m[0:3] == b'foo'
    f.close()
    try:
        m.resize(512)
    except SystemError:
        pass
    else:

        assert len(m) == 512

        try:
            m.seek(513, 0)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

        assert m[0:3] == b'foo'
        f = open(TESTFN, 'rb')
        f.seek(0, 2)

        assert f.tell() == halfsize + 512
        f.close()

        assert m.size() == halfsize + 512
    m.close()
finally:
    f.close()
    try:
        os.unlink(TESTFN)
    except OSError:
        pass
print("MmapTests::test_offset: ok")
