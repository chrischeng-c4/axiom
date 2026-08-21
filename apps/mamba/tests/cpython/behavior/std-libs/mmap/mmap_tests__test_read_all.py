# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_read_all"
# subject = "cpython.test_mmap.MmapTests.test_read_all"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_read_all
"""Auto-ported test: MmapTests::test_read_all (CPython 3.12 oracle)."""


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
m = mmap.mmap(-1, 16)
pass
m.write(bytes(range(16)))
m.seek(0)

assert m.read() == bytes(range(16))
m.seek(8)

assert m.read() == bytes(range(8, 16))
m.seek(16)

assert m.read() == b''
m.seek(3)

assert m.read(None) == bytes(range(3, 16))
m.seek(4)

assert m.read(-1) == bytes(range(4, 16))
m.seek(5)

assert m.read(-2) == bytes(range(5, 16))
m.seek(9)

assert m.read(-42) == bytes(range(9, 16))
print("MmapTests::test_read_all: ok")
