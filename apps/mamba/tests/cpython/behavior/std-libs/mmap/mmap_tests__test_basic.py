# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_basic"
# subject = "cpython.test_mmap.MmapTests.test_basic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_basic
"""Auto-ported test: MmapTests::test_basic (CPython 3.12 oracle)."""


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
f = open(TESTFN, 'bw+')
try:
    f.write(b'\x00' * PAGESIZE)
    f.write(b'foo')
    f.write(b'\x00' * (PAGESIZE - 3))
    f.flush()
    m = mmap.mmap(f.fileno(), 2 * PAGESIZE)
finally:
    f.close()
tp = str(type(m))

assert m.find(b'foo') == PAGESIZE

assert len(m) == 2 * PAGESIZE

assert m[0] == 0

assert m[0:3] == b'\x00\x00\x00'

try:
    m.__getitem__(len(m))
    raise AssertionError('expected IndexError')
except IndexError:
    pass

try:
    m.__setitem__(len(m), b'\x00')
    raise AssertionError('expected IndexError')
except IndexError:
    pass
m[0] = b'3'[0]
m[PAGESIZE + 3:PAGESIZE + 3 + 3] = b'bar'

assert m[0] == b'3'[0]

assert m[0:3] == b'3\x00\x00'

assert m[PAGESIZE - 1:PAGESIZE + 7] == b'\x00foobar\x00'
m.flush()
match = re.search(b'[A-Za-z]+', m)
if match is None:

    raise AssertionError('regex match on mmap failed!')
else:
    start, end = match.span(0)
    length = end - start

    assert start == PAGESIZE

    assert end == PAGESIZE + 6
m.seek(0, 0)

assert m.tell() == 0
m.seek(42, 1)

assert m.tell() == 42
m.seek(0, 2)

assert m.tell() == len(m)

try:
    m.seek(-1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    m.seek(1, 2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    m.seek(-len(m) - 1, 2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
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
    f = open(TESTFN, 'rb')
    try:
        f.seek(0, 2)

        assert f.tell() == 512
    finally:
        f.close()

    assert m.size() == 512
m.close()
print("MmapTests::test_basic: ok")
