# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile64"
# dimension = "behavior"
# case = "tests_with_source_file__test_stored"
# subject = "cpython.test_zipfile64.TestsWithSourceFile.testStored"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipfile64.py::TestsWithSourceFile::testStored
"""Auto-ported test: TestsWithSourceFile::testStored (CPython 3.12 oracle)."""


from test import support
import zipfile, unittest
import time
import sys
from tempfile import TemporaryFile
from test.support import os_helper
from test.support import requires_zlib


support.requires('extralargefile', 'test requires loads of disk-space bytes and a long time to run')

TESTFN = os_helper.TESTFN

TESTFN2 = TESTFN + '2'

_PRINT_WORKING_MSG_INTERVAL = 60


# --- test body ---
def zipTest(f, compression):
    with zipfile.ZipFile(f, 'w', compression) as zipfp:
        filecount = 6 * 1024 ** 3 // len(self_data)
        next_time = time.monotonic() + _PRINT_WORKING_MSG_INTERVAL
        for num in range(filecount):
            zipfp.writestr('testfn%d' % num, self_data)
            if next_time <= time.monotonic():
                next_time = time.monotonic() + _PRINT_WORKING_MSG_INTERVAL
                print('  zipTest still writing %d of %d, be patient...' % (num, filecount), file=sys.__stdout__)
                sys.__stdout__.flush()
    with zipfile.ZipFile(f, 'r', compression) as zipfp:
        for num in range(filecount):

            assert zipfp.read('testfn%d' % num) == self_data
            if next_time <= time.monotonic():
                next_time = time.monotonic() + _PRINT_WORKING_MSG_INTERVAL
                print('  zipTest still reading %d of %d, be patient...' % (num, filecount), file=sys.__stdout__)
                sys.__stdout__.flush()

        assert zipfp.testzip() is None
line_gen = ('Test of zipfile line %d.' % i for i in range(1000000))
self_data = '\n'.join(line_gen).encode('ascii')
with TemporaryFile() as f:
    zipTest(f, zipfile.ZIP_STORED)

    assert not f.closed
zipTest(TESTFN2, zipfile.ZIP_STORED)
print("TestsWithSourceFile::testStored: ok")
