# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bufio"
# dimension = "behavior"
# case = "py_buffer_size_test__test_nullpat"
# subject = "cpython.test_bufio.PyBufferSizeTest.test_nullpat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bufio.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bufio.py::PyBufferSizeTest::test_nullpat
"""Auto-ported test: PyBufferSizeTest::test_nullpat (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import io
import _pyio as pyio


lengths = list(range(1, 257)) + [512, 1000, 1024, 2048, 4096, 8192, 10000, 16384, 32768, 65536, 1000000]


# --- test body ---
open = staticmethod(pyio.open)

def drive_one(pattern):
    for length in lengths:
        q, r = divmod(length, len(pattern))
        teststring = pattern * q + pattern[:r]

        assert len(teststring) == length
        try_one(teststring)
        try_one(teststring + b'x')
        try_one(teststring[:-1])

def try_one(s):
    os_helper.unlink(os_helper.TESTFN)
    f = open(os_helper.TESTFN, 'wb')
    try:
        f.write(s)
        f.write(b'\n')
        f.write(s)
        f.close()
        f = open(os_helper.TESTFN, 'rb')
        line = f.readline()

        assert line == s + b'\n'
        line = f.readline()

        assert line == s
        line = f.readline()

        assert not line
        f.close()
    finally:
        os_helper.unlink(os_helper.TESTFN)
drive_one(b'\x00' * 1000)
print("PyBufferSizeTest::test_nullpat: ok")
