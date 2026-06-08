# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multibytecodec"
# dimension = "behavior"
# case = "test_incremental_decoder__test_iso2022"
# subject = "cpython.test_multibytecodec.Test_IncrementalDecoder.test_iso2022"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multibytecodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_multibytecodec.py::Test_IncrementalDecoder::test_iso2022
"""Auto-ported test: Test_IncrementalDecoder::test_iso2022 (CPython 3.12 oracle)."""


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
decoder = codecs.getincrementaldecoder('iso2022-jp')()
ESC = b'\x1b'

assert decoder.decode(ESC + b'(') == ''

assert decoder.decode(b'B', True) == ''

assert decoder.decode(ESC + b'$') == ''

assert decoder.decode(b'B@$') == '世'

assert decoder.decode(b'@$@') == '世'

assert decoder.decode(b'$', True) == '世'

assert decoder.reset() == None

assert decoder.decode(b'@$') == '@$'

assert decoder.decode(ESC + b'$') == ''

try:
    decoder.decode(b'', True)
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError:
    pass

assert decoder.decode(b'B@$') == '世'
print("Test_IncrementalDecoder::test_iso2022: ok")
