# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multibytecodec"
# dimension = "behavior"
# case = "test_incremental_encoder__test_state_methods_with_non_buffer_state"
# subject = "cpython.test_multibytecodec.Test_IncrementalEncoder.test_state_methods_with_non_buffer_state"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multibytecodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_multibytecodec.py::Test_IncrementalEncoder::test_state_methods_with_non_buffer_state
"""Auto-ported test: Test_IncrementalEncoder::test_state_methods_with_non_buffer_state (CPython 3.12 oracle)."""


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
encoder = codecs.getincrementalencoder('iso2022_jp')()

assert encoder.encode('z') == b'z'
en_state = encoder.getstate()

assert encoder.encode('あ') == b'\x1b$B$"'
jp_state = encoder.getstate()

assert encoder.encode('z') == b'\x1b(Bz'
encoder.setstate(jp_state)

assert encoder.encode('あ') == b'$"'
encoder.setstate(en_state)

assert encoder.encode('z') == b'z'
print("Test_IncrementalEncoder::test_state_methods_with_non_buffer_state: ok")
