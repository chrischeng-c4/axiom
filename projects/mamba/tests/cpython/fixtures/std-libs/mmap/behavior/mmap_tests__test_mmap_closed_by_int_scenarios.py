# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_mmap_closed_by_int_scenarios"
# subject = "cpython.test_mmap.MmapTests.test_mmap_closed_by_int_scenarios"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mmap.py::MmapTests::test_mmap_closed_by_int_scenarios
"""Auto-ported test: MmapTests::test_mmap_closed_by_int_scenarios (CPython 3.12 oracle)."""


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
'\n        gh-103987: Test that mmap objects raise ValueError\n                for closed mmap files\n        '

class MmapClosedByIntContext:

    def __init__(self, access) -> None:
        self.access = access

    def __enter__(self):
        self.f = open(TESTFN, 'w+b')
        self.f.write(random.randbytes(100))
        self.f.flush()
        m = mmap.mmap(self.f.fileno(), 100, access=self.access)

        class X:

            def __index__(self):
                m.close()
                return 10
        return (m, X)

    def __exit__(self, exc_type, exc_value, traceback):
        self.f.close()
read_access_modes = [mmap.ACCESS_READ, mmap.ACCESS_WRITE, mmap.ACCESS_COPY, mmap.ACCESS_DEFAULT]
write_access_modes = [mmap.ACCESS_WRITE, mmap.ACCESS_COPY, mmap.ACCESS_DEFAULT]
for access in read_access_modes:
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m[X()]
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m[X():20]
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m[X():20:2]
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m[20:X():-2]
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m.read(X())
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m.find(b'1', 1, X())
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
for access in write_access_modes:
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m[X():20] = b'1' * 10
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m[X():20:2] = b'1' * 5
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m[20:X():-2] = b'1' * 5
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m.move(1, 2, X())
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
    with MmapClosedByIntContext(access) as (m, X):
        try:
            m.write_byte(X())
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('mmap closed or invalid', str(_aR_e))
print("MmapTests::test_mmap_closed_by_int_scenarios: ok")
