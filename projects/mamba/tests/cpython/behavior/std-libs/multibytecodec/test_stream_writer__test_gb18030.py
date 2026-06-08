# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multibytecodec"
# dimension = "behavior"
# case = "test_stream_writer__test_gb18030"
# subject = "cpython.test_multibytecodec.Test_StreamWriter.test_gb18030"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multibytecodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_multibytecodec.py::Test_StreamWriter::test_gb18030
"""Auto-ported test: Test_StreamWriter::test_gb18030 (CPython 3.12 oracle)."""


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
s = io.BytesIO()
c = codecs.getwriter('gb18030')(s)
c.write('123')

assert s.getvalue() == b'123'
c.write('𒍅')

assert s.getvalue() == b'123\x907\x959'
c.write('가¬')

assert s.getvalue() == b'123\x907\x959\x827\xcf5\x810\x851'
print("Test_StreamWriter::test_gb18030: ok")
