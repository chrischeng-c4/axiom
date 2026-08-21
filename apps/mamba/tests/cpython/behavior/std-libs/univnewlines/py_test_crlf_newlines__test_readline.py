# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "univnewlines"
# dimension = "behavior"
# case = "py_test_crlf_newlines__test_readline"
# subject = "cpython.test_univnewlines.PyTestCRLFNewlines.test_readline"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_univnewlines.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_univnewlines.py::PyTestCRLFNewlines::test_readline
"""Auto-ported test: PyTestCRLFNewlines::test_readline (CPython 3.12 oracle)."""


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
open = staticmethod(pyio.open)
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
    data = []
    d = fp.readline()
    while d:
        data.append(d)
        d = fp.readline()

assert data == DATA_SPLIT

assert repr(fp.newlines) == repr(NEWLINE)
print("PyTestCRLFNewlines::test_readline: ok")
