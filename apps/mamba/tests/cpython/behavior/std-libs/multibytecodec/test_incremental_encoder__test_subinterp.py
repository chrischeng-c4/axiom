# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multibytecodec"
# dimension = "behavior"
# case = "test_incremental_encoder__test_subinterp"
# subject = "cpython.test_multibytecodec.Test_IncrementalEncoder.test_subinterp"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_multibytecodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_multibytecodec.py::Test_IncrementalEncoder::test_subinterp
"""Auto-ported test: Test_IncrementalEncoder::test_subinterp (CPython 3.12 oracle)."""


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
import _testcapi
encoding = 'cp932'
text = 'Python の開発は、1990 年ごろから開始されています。'
code = textwrap.dedent('\n            import codecs\n            encoding = %r\n            text = %r\n            encoder = codecs.getincrementalencoder(encoding)()\n            text2 = encoder.encode(text).decode(encoding)\n            if text2 != text:\n                raise ValueError(f"encoding issue: {text2!a} != {text!a}")\n        ') % (encoding, text)
res = _testcapi.run_in_subinterp(code)

assert res == 0
print("Test_IncrementalEncoder::test_subinterp: ok")
