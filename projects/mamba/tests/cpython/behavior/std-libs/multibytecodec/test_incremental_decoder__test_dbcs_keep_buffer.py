# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multibytecodec"
# dimension = "behavior"
# case = "test_incremental_decoder__test_dbcs_keep_buffer"
# subject = "cpython.test_multibytecodec.Test_IncrementalDecoder.test_dbcs_keep_buffer"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multibytecodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_multibytecodec.py::Test_IncrementalDecoder::test_dbcs_keep_buffer
"""Auto-ported test: Test_IncrementalDecoder::test_dbcs_keep_buffer (CPython 3.12 oracle)."""


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
decoder = codecs.getincrementaldecoder('cp949')()

assert decoder.decode(b'\xc6\xc4\xc0') == '파'

try:
    decoder.decode(b'', True)
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError:
    pass

assert decoder.decode(b'\xcc') == '이'

assert decoder.decode(b'\xc6\xc4\xc0') == '파'

try:
    decoder.decode(b'\xcc\xbd', True)
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError:
    pass

assert decoder.decode(b'\xcc') == '이'
print("Test_IncrementalDecoder::test_dbcs_keep_buffer: ok")
