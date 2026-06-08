# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "behavior"
# case = "aifc_low_level_test__test_write_aiff_by_extension"
# subject = "cpython.test_aifc.AIFCLowLevelTest.test_write_aiff_by_extension"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_aifc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_aifc.py::AIFCLowLevelTest::test_write_aiff_by_extension
"""Auto-ported test: AIFCLowLevelTest::test_write_aiff_by_extension (CPython 3.12 oracle)."""


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
sampwidth = 2
filename = TESTFN + '.aiff'
fout = self_fout = aifc.open(filename, 'wb')
pass
fout.setparams((1, sampwidth, 1, 1, b'ULAW', b''))
frames = b'\x00' * fout.getnchannels() * sampwidth
fout.writeframes(frames)
fout.close()
f = self_f = aifc.open(filename, 'rb')

assert f.getcomptype() == b'NONE'
f.close()
print("AIFCLowLevelTest::test_write_aiff_by_extension: ok")
