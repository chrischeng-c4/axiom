# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_extended_set_del_slice"
# subject = "cpython.test_mmap.MmapTests.test_extended_set_del_slice"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_extended_set_del_slice
"""Auto-ported test: MmapTests::test_extended_set_del_slice (CPython 3.12 oracle)."""


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
s = bytes(reversed(range(256)))
m = mmap.mmap(-1, len(s))
indices = (0, None, 1, 3, 19, 300, sys.maxsize, -1, -2, -31, -300)
for start in indices:
    for stop in indices:
        for step in indices[1:]:
            m[:] = s

            assert m[:] == s
            L = list(s)
            data = L[start:stop:step]
            data = bytes(reversed(data))
            L[start:stop:step] = data
            m[start:stop:step] = data

            assert m[:] == bytes(L)
print("MmapTests::test_extended_set_del_slice: ok")
