# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "large_mmap_tests__test_large_filesize"
# subject = "cpython.test_mmap.LargeMmapTests.test_large_filesize"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::LargeMmapTests::test_large_filesize
"""Auto-ported test: LargeMmapTests::test_large_filesize (CPython 3.12 oracle)."""


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
def _make_test_file(num_zeroes, tail):
    if sys.platform[:3] == 'win' or sys.platform == 'darwin':
        requires('largefile', 'test requires %s bytes and a long time to run' % str(6442450944))
    f = open(TESTFN, 'w+b')
    try:
        f.seek(num_zeroes)
        f.write(tail)
        f.flush()
    except (OSError, OverflowError, ValueError):
        try:
            f.close()
        except (OSError, OverflowError):
            pass
        raise unittest.SkipTest('filesystem does not have largefile support')
    return f

def _test_around_boundary(boundary):
    tail = b'  DEARdear  '
    start = boundary - len(tail) // 2
    end = start + len(tail)
    with _make_test_file(start, tail) as f:
        with mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ) as m:

            assert m[start:end] == tail
unlink(TESTFN)
with _make_test_file(6442450943, b' ') as f:
    if sys.maxsize < 6442450944:
        try:
            mmap.mmap(f.fileno(), 6442450944, access=mmap.ACCESS_READ)
            raise AssertionError('expected OverflowError')
        except OverflowError:
            pass
        try:
            mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
    with mmap.mmap(f.fileno(), 65536, access=mmap.ACCESS_READ) as m:

        assert m.size() == 6442450944
print("LargeMmapTests::test_large_filesize: ok")
