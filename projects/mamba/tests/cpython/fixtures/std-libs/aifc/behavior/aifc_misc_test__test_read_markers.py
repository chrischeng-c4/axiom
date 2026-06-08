# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "behavior"
# case = "aifc_misc_test__test_read_markers"
# subject = "cpython.test_aifc.AifcMiscTest.test_read_markers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_aifc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_aifc.py::AifcMiscTest::test_read_markers
"""Auto-ported test: AifcMiscTest::test_read_markers (CPython 3.12 oracle)."""


from test.support import findfile
from test.support.os_helper import TESTFN, unlink
from test.support.warnings_helper import check_no_resource_warning, import_deprecated
import unittest
from unittest import mock
from test import audiotests
import io
import sys
import struct


aifc = import_deprecated('aifc')

audioop = import_deprecated('audioop')

class AifcTest(audiotests.AudioWriteTests, audiotests.AudioTestsWithSourceFile):
    module = aifc
    close_fd = True
    test_unseekable_read = None


# --- test body ---
fout = self_fout = aifc.open(TESTFN, 'wb')
fout.aiff()
fout.setparams((1, 1, 1, 1, b'NONE', b''))
fout.setmark(1, 0, b'odd')
fout.setmark(2, 0, b'even')
fout.writeframes(b'\x00')
fout.close()
f = aifc.open(TESTFN, 'rb')
pass

assert f.getmarkers() == [(1, 0, b'odd'), (2, 0, b'even')]

assert f.getmark(1) == (1, 0, b'odd')

assert f.getmark(2) == (2, 0, b'even')

try:
    f.getmark(3)
    raise AssertionError('expected aifc.Error')
except aifc.Error:
    pass
print("AifcMiscTest::test_read_markers: ok")
