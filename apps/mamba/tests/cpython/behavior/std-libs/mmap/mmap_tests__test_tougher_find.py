# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_tougher_find"
# subject = "cpython.test_mmap.MmapTests.test_tougher_find"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_tougher_find
"""Auto-ported test: MmapTests::test_tougher_find (CPython 3.12 oracle)."""


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
    data = b'aabaac\x00deef\x00\x00aa\x00'
    n = len(data)
    f.write(data)
    f.flush()
    m = mmap.mmap(f.fileno(), n)
for start in range(n + 1):
    for finish in range(start, n + 1):
        slice = data[start:finish]

        assert m.find(slice) == data.find(slice)

        assert m.find(slice + b'x') == -1
m.close()
print("MmapTests::test_tougher_find: ok")
