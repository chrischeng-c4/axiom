# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "normalize_test__test_euc_encoding"
# subject = "cpython.test_locale.NormalizeTest.test_euc_encoding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locale.py::NormalizeTest::test_euc_encoding
"""Auto-ported test: NormalizeTest::test_euc_encoding (CPython 3.12 oracle)."""


from decimal import Decimal
from test.support import verbose, is_android, is_emscripten, is_wasi
from test.support.warnings_helper import check_warnings
import unittest
import locale
import sys
import codecs


class BaseFormattingTest(object):

    def _test_format_string(self, format, value, out, **format_opts):
        self.assertEqual(locale.format_string(format, value, **format_opts), out)

    def _test_currency(self, value, out, **format_opts):
        self.assertEqual(locale.currency(value, **format_opts), out)


# --- test body ---
def check(localename, expected):

    assert locale.normalize(localename) == expected
check('ja_jp.euc', 'ja_JP.eucJP')
check('ja_jp.eucjp', 'ja_JP.eucJP')
check('ko_kr.euc', 'ko_KR.eucKR')
check('ko_kr.euckr', 'ko_KR.eucKR')
check('zh_cn.euc', 'zh_CN.eucCN')
check('zh_tw.euc', 'zh_TW.eucTW')
check('zh_tw.euctw', 'zh_TW.eucTW')
print("NormalizeTest::test_euc_encoding: ok")
