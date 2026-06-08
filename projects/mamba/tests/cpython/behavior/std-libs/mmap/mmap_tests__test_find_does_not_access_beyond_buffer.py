# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_find_does_not_access_beyond_buffer"
# subject = "cpython.test_mmap.MmapTests.test_find_does_not_access_beyond_buffer"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_find_does_not_access_beyond_buffer
"""Auto-ported test: MmapTests::test_find_does_not_access_beyond_buffer (CPython 3.12 oracle)."""


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
try:
    flags = mmap.MAP_PRIVATE | mmap.MAP_ANONYMOUS
    PAGESIZE = mmap.PAGESIZE
    PROT_NONE = 0
    PROT_READ = mmap.PROT_READ
except AttributeError as e:
    raise unittest.SkipTest('mmap flags unavailable') from e
for i in range(0, 2049):
    with mmap.mmap(-1, PAGESIZE * (i + 1), flags=flags, prot=PROT_NONE) as guard:
        with mmap.mmap(-1, PAGESIZE * (i + 2048), flags=flags, prot=PROT_READ) as fm:
            fm.find(b'fo', -2)
print("MmapTests::test_find_does_not_access_beyond_buffer: ok")
