# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "behavior"
# case = "aifc_misc_test__test_params_added"
# subject = "cpython.test_aifc.AifcMiscTest.test_params_added"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_aifc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_aifc.py::AifcMiscTest::test_params_added
"""Auto-ported test: AifcMiscTest::test_params_added (CPython 3.12 oracle)."""


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
f = self_f = aifc.open(TESTFN, 'wb')
f.aiff()
f.setparams((1, 1, 1, 1, b'NONE', b''))
f.close()
f = aifc.open(TESTFN, 'rb')
pass
params = f.getparams()

assert params.nchannels == f.getnchannels()

assert params.sampwidth == f.getsampwidth()

assert params.framerate == f.getframerate()

assert params.nframes == f.getnframes()

assert params.comptype == f.getcomptype()

assert params.compname == f.getcompname()
print("AifcMiscTest::test_params_added: ok")
