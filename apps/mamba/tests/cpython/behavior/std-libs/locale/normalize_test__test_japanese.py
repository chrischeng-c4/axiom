# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "normalize_test__test_japanese"
# subject = "cpython.test_locale.NormalizeTest.test_japanese"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locale.py::NormalizeTest::test_japanese
"""Auto-ported test: NormalizeTest::test_japanese (CPython 3.12 oracle)."""


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
check('ja', 'ja_JP.eucJP')
check('ja.jis', 'ja_JP.JIS7')
check('ja.sjis', 'ja_JP.SJIS')
check('ja_jp', 'ja_JP.eucJP')
check('ja_jp.ajec', 'ja_JP.eucJP')
check('ja_jp.euc', 'ja_JP.eucJP')
check('ja_jp.eucjp', 'ja_JP.eucJP')
check('ja_jp.iso-2022-jp', 'ja_JP.JIS7')
check('ja_jp.iso2022jp', 'ja_JP.JIS7')
check('ja_jp.jis', 'ja_JP.JIS7')
check('ja_jp.jis7', 'ja_JP.JIS7')
check('ja_jp.mscode', 'ja_JP.SJIS')
check('ja_jp.pck', 'ja_JP.SJIS')
check('ja_jp.sjis', 'ja_JP.SJIS')
check('ja_jp.ujis', 'ja_JP.eucJP')
check('ja_jp.utf8', 'ja_JP.UTF-8')
check('japan', 'ja_JP.eucJP')
check('japanese', 'ja_JP.eucJP')
check('japanese-euc', 'ja_JP.eucJP')
check('japanese.euc', 'ja_JP.eucJP')
check('japanese.sjis', 'ja_JP.SJIS')
check('jp_jp', 'ja_JP.eucJP')
print("NormalizeTest::test_japanese: ok")
