# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multibytecodec"
# dimension = "behavior"
# case = "test_incremental_decoder__test_state_methods"
# subject = "cpython.test_multibytecodec.Test_IncrementalDecoder.test_state_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multibytecodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_multibytecodec.py::Test_IncrementalDecoder::test_state_methods
"""Auto-ported test: Test_IncrementalDecoder::test_state_methods (CPython 3.12 oracle)."""


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

assert decoder.decode(b'\xa4\xa6') == 'う'
pending1, _ = decoder.getstate()

assert pending1 == b''

assert decoder.decode(b'\xa4') == ''
pending2, flags2 = decoder.getstate()

assert pending2 == b'\xa4'

assert decoder.decode(b'\xa6') == 'う'
pending3, _ = decoder.getstate()

assert pending3 == b''
decoder.setstate((pending2, flags2))

assert decoder.decode(b'\xa6') == 'う'
pending4, _ = decoder.getstate()

assert pending4 == b''
decoder.setstate((b'abc', 123456789))

assert decoder.getstate() == (b'abc', 123456789)
print("Test_IncrementalDecoder::test_state_methods: ok")
