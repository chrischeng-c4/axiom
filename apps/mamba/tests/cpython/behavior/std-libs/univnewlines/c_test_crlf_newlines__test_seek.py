# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "univnewlines"
# dimension = "behavior"
# case = "c_test_crlf_newlines__test_seek"
# subject = "cpython.test_univnewlines.CTestCRLFNewlines.test_seek"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_univnewlines.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_univnewlines.py::CTestCRLFNewlines::test_seek
"""Auto-ported test: CTestCRLFNewlines::test_seek (CPython 3.12 oracle)."""


import io
import _pyio as pyio
import unittest
import os
import sys
from test.support import os_helper


if not hasattr(sys.stdin, 'newlines'):
    raise unittest.SkipTest('This Python does not have universal newline support')

FATX = 'x' * 2 ** 14

DATA_TEMPLATE = ['line1=1', "line2='this is a very long line designed to go past any default " + 'buffer limits that exist in io.py but we also want to test ' + "the uncommon case, naturally.'", 'def line3():pass', "line4 = '%s'" % FATX]

DATA_LF = '\n'.join(DATA_TEMPLATE) + '\n'

DATA_CR = '\r'.join(DATA_TEMPLATE) + '\r'

DATA_CRLF = '\r\n'.join(DATA_TEMPLATE) + '\r\n'

DATA_MIXED = '\n'.join(DATA_TEMPLATE) + '\r'

DATA_SPLIT = [x + '\n' for x in DATA_TEMPLATE]

class CTest:
    open = io.open

class PyTest:
    open = staticmethod(pyio.open)


# --- test body ---
open = io.open
READMODE = 'r'
WRITEMODE = 'wb'
NEWLINE = '\r\n'
DATA = DATA_CRLF
data = DATA
if 'b' in WRITEMODE:
    data = data.encode('ascii')
with open(os_helper.TESTFN, WRITEMODE) as fp:
    fp.write(data)
with open(os_helper.TESTFN, READMODE) as fp:
    fp.readline()
    pos = fp.tell()
    data = fp.readlines()

    assert data == DATA_SPLIT[1:]
    fp.seek(pos)
    data = fp.readlines()

assert data == DATA_SPLIT[1:]
print("CTestCRLFNewlines::test_seek: ok")
