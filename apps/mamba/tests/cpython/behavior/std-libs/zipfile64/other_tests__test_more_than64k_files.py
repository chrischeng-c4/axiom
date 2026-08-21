# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile64"
# dimension = "behavior"
# case = "other_tests__test_more_than64k_files"
# subject = "cpython.test_zipfile64.OtherTests.testMoreThan64kFiles"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipfile64.py::OtherTests::testMoreThan64kFiles
"""Auto-ported test: OtherTests::testMoreThan64kFiles (CPython 3.12 oracle)."""


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
with zipfile.ZipFile(TESTFN, mode='w', allowZip64=True) as zipf:
    zipf.debug = 100
    numfiles = (1 << 16) * 3 // 2
    for i in range(numfiles):
        zipf.writestr('foo%08d' % i, '%d' % (i ** 3 % 57))

    assert len(zipf.namelist()) == numfiles
with zipfile.ZipFile(TESTFN, mode='r') as zipf2:

    assert len(zipf2.namelist()) == numfiles
    for i in range(numfiles):
        content = zipf2.read('foo%08d' % i).decode('ascii')

        assert content == '%d' % (i ** 3 % 57)
print("OtherTests::testMoreThan64kFiles: ok")
