# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multibytecodec"
# dimension = "behavior"
# case = "test_hz_stateful__test_incrementalencoder_final"
# subject = "cpython.test_multibytecodec.TestHZStateful.test_incrementalencoder_final"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multibytecodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_multibytecodec.py::TestHZStateful::test_incrementalencoder_final
"""Auto-ported test: TestHZStateful::test_incrementalencoder_final (CPython 3.12 oracle)."""


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
text = '世世'
encoding = 'iso-2022-jp'
expected = b'\x1b$B@$@$'
reset = b'\x1b(B'
expected_reset = expected + reset
text = '聊聊'
encoding = 'hz'
expected = b'~{ADAD'
reset = b'~}'
expected_reset = expected + reset
encoder = codecs.getincrementalencoder(encoding)()
last_index = len(text) - 1
output = b''.join((encoder.encode(char, index == last_index) for index, char in enumerate(text)))

assert output == expected_reset

assert encoder.encode('', final=True) == b''
print("TestHZStateful::test_incrementalencoder_final: ok")
