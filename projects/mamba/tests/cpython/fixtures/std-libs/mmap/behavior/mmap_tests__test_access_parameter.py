# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_access_parameter"
# subject = "cpython.test_mmap.MmapTests.test_access_parameter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_access_parameter
"""Auto-ported test: MmapTests::test_access_parameter (CPython 3.12 oracle)."""


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
mapsize = 10
with open(TESTFN, 'wb') as fp:
    fp.write(b'a' * mapsize)
with open(TESTFN, 'rb') as f:
    m = mmap.mmap(f.fileno(), mapsize, access=mmap.ACCESS_READ)

    assert m[:] == b'a' * mapsize
    try:
        m[:] = b'b' * mapsize
    except TypeError:
        pass
    else:

        raise AssertionError('Able to write to readonly memory map')
    try:
        m[0] = b'b'
    except TypeError:
        pass
    else:

        raise AssertionError('Able to write to readonly memory map')
    try:
        m.seek(0, 0)
        m.write(b'abc')
    except TypeError:
        pass
    else:

        raise AssertionError('Able to write to readonly memory map')
    try:
        m.seek(0, 0)
        m.write_byte(b'd')
    except TypeError:
        pass
    else:

        raise AssertionError('Able to write to readonly memory map')
    try:
        m.resize(2 * mapsize)
    except SystemError:
        pass
    except TypeError:
        pass
    else:

        raise AssertionError('Able to resize readonly memory map')
    with open(TESTFN, 'rb') as fp:

        assert fp.read() == b'a' * mapsize
with open(TESTFN, 'r+b') as f:
    try:
        m = mmap.mmap(f.fileno(), mapsize + 1)
    except ValueError:
        if sys.platform.startswith('win'):

            raise AssertionError('Opening mmap with size+1 should work on Windows.')
    else:
        if not sys.platform.startswith('win'):

            raise AssertionError('Opening mmap with size+1 should raise ValueError.')
        m.close()
    if sys.platform.startswith('win'):
        with open(TESTFN, 'r+b') as f:
            f.truncate(mapsize)
with open(TESTFN, 'r+b') as f:
    m = mmap.mmap(f.fileno(), mapsize, access=mmap.ACCESS_WRITE)
    m[:] = b'c' * mapsize

    assert m[:] == b'c' * mapsize
    m.flush()
    m.close()
with open(TESTFN, 'rb') as f:
    stuff = f.read()

assert stuff == b'c' * mapsize
with open(TESTFN, 'r+b') as f:
    m = mmap.mmap(f.fileno(), mapsize, access=mmap.ACCESS_COPY)
    m[:] = b'd' * mapsize

    assert m[:] == b'd' * mapsize
    m.flush()
    with open(TESTFN, 'rb') as fp:

        assert fp.read() == b'c' * mapsize

    try:
        m.resize(2 * mapsize)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    m.close()
with open(TESTFN, 'r+b') as f:

    try:
        mmap.mmap(f.fileno(), mapsize, access=4)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
if os.name == 'posix':
    with open(TESTFN, 'r+b') as f:

        try:
            mmap.mmap(f.fileno(), mapsize, flags=mmap.MAP_PRIVATE, prot=mmap.PROT_READ, access=mmap.ACCESS_WRITE)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
    prot = mmap.PROT_READ | getattr(mmap, 'PROT_EXEC', 0)
    with open(TESTFN, 'r+b') as f:
        try:
            m = mmap.mmap(f.fileno(), mapsize, prot=prot)
        except PermissionError:
            pass
        else:

            try:
                m.write(b'abcdef')
                raise AssertionError('expected TypeError')
            except TypeError:
                pass

            try:
                m.write_byte(0)
                raise AssertionError('expected TypeError')
            except TypeError:
                pass
            m.close()
print("MmapTests::test_access_parameter: ok")
