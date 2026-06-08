# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multibytecodec"
# dimension = "behavior"
# case = "test_stream_reader__test_bug1728403"
# subject = "cpython.test_multibytecodec.Test_StreamReader.test_bug1728403"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multibytecodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_multibytecodec.py::Test_StreamReader::test_bug1728403
"""Auto-ported test: Test_StreamReader::test_bug1728403 (CPython 3.12 oracle)."""


import _multibytecodec
import codecs
import io
import sys
import textwrap
import unittest
from test import support
from test.support import os_helper
from test.support.os_helper import TESTFN


ALL_CJKENCODINGS = ['gb2312', 'gbk', 'gb18030', 'hz', 'big5hkscs', 'cp932', 'shift_jis', 'euc_jp', 'euc_jisx0213', 'shift_jisx0213', 'euc_jis_2004', 'shift_jis_2004', 'cp949', 'euc_kr', 'johab', 'big5', 'cp950', 'iso2022_jp', 'iso2022_jp_1', 'iso2022_jp_2', 'iso2022_jp_2004', 'iso2022_jp_3', 'iso2022_jp_ext', 'iso2022_kr']


# --- test body ---
try:
    f = open(TESTFN, 'wb')
    try:
        f.write(b'\xa1')
    finally:
        f.close()
    f = codecs.open(TESTFN, encoding='cp949')
    try:

        try:
            f.read(2)
            raise AssertionError('expected UnicodeDecodeError')
        except UnicodeDecodeError:
            pass
    finally:
        f.close()
finally:
    os_helper.unlink(TESTFN)
print("Test_StreamReader::test_bug1728403: ok")
