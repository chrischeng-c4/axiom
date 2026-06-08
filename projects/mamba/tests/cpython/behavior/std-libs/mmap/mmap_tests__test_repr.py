# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_repr"
# subject = "cpython.test_mmap.MmapTests.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_repr
"""Auto-ported test: MmapTests::test_repr (CPython 3.12 oracle)."""


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
open_mmap_repr_pat = re.compile('<mmap.mmap closed=False, access=(?P<access>\\S+), length=(?P<length>\\d+), pos=(?P<pos>\\d+), offset=(?P<offset>\\d+)>')
closed_mmap_repr_pat = re.compile('<mmap.mmap closed=True>')
mapsizes = (50, 100, 1000, 1000000, 10000000)
offsets = tuple((mapsize // 2 // mmap.ALLOCATIONGRANULARITY * mmap.ALLOCATIONGRANULARITY for mapsize in mapsizes))
for offset, mapsize in zip(offsets, mapsizes):
    data = b'a' * mapsize
    length = mapsize - offset
    accesses = ('ACCESS_DEFAULT', 'ACCESS_READ', 'ACCESS_COPY', 'ACCESS_WRITE')
    positions = (0, length // 10, length // 5, length // 4)
    with open(TESTFN, 'wb+') as fp:
        fp.write(data)
        fp.flush()
        for access, pos in itertools.product(accesses, positions):
            accint = getattr(mmap, access)
            with mmap.mmap(fp.fileno(), length, access=accint, offset=offset) as mm:
                mm.seek(pos)
                match = open_mmap_repr_pat.match(repr(mm))

                assert match is not None

                assert match.group('access') == access

                assert match.group('length') == str(length)

                assert match.group('pos') == str(pos)

                assert match.group('offset') == str(offset)
            match = closed_mmap_repr_pat.match(repr(mm))

            assert match is not None
print("MmapTests::test_repr: ok")
