# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bufio"
# dimension = "behavior"
# case = "c_buffer_size_test__test_primepat"
# subject = "cpython.test_bufio.CBufferSizeTest.test_primepat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bufio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bufio.py::CBufferSizeTest::test_primepat
"""Auto-ported test: CBufferSizeTest::test_primepat (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import io
import _pyio as pyio


lengths = list(range(1, 257)) + [512, 1000, 1024, 2048, 4096, 8192, 10000, 16384, 32768, 65536, 1000000]


# --- test body ---
open = io.open

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
drive_one(b'1234567890\x00\x01\x02\x03\x04\x05\x06')
print("CBufferSizeTest::test_primepat: ok")
