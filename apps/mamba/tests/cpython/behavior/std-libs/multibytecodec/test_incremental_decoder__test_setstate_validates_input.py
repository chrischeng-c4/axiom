# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multibytecodec"
# dimension = "behavior"
# case = "test_incremental_decoder__test_setstate_validates_input"
# subject = "cpython.test_multibytecodec.Test_IncrementalDecoder.test_setstate_validates_input"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multibytecodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_multibytecodec.py::Test_IncrementalDecoder::test_setstate_validates_input
"""Auto-ported test: Test_IncrementalDecoder::test_setstate_validates_input (CPython 3.12 oracle)."""


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
decoder = codecs.getincrementaldecoder('euc_jp')()

try:
    decoder.setstate(123)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    decoder.setstate(('invalid', 0))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    decoder.setstate((b'1234', 'invalid'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    decoder.setstate((b'123456789', 0))
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass
print("Test_IncrementalDecoder::test_setstate_validates_input: ok")
