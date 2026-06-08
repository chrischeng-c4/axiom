# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "normalize_test__test_hyphenated_encoding"
# subject = "cpython.test_locale.NormalizeTest.test_hyphenated_encoding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locale.py::NormalizeTest::test_hyphenated_encoding
"""Auto-ported test: NormalizeTest::test_hyphenated_encoding (CPython 3.12 oracle)."""


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
check('az_AZ.iso88599e', 'az_AZ.ISO8859-9E')
check('az_AZ.ISO8859-9E', 'az_AZ.ISO8859-9E')
check('tt_RU.koi8c', 'tt_RU.KOI8-C')
check('tt_RU.KOI8-C', 'tt_RU.KOI8-C')
check('lo_LA.cp1133', 'lo_LA.IBM-CP1133')
check('lo_LA.ibmcp1133', 'lo_LA.IBM-CP1133')
check('lo_LA.IBM-CP1133', 'lo_LA.IBM-CP1133')
check('uk_ua.microsoftcp1251', 'uk_UA.CP1251')
check('uk_ua.microsoft-cp1251', 'uk_UA.CP1251')
check('ka_ge.georgianacademy', 'ka_GE.GEORGIAN-ACADEMY')
check('ka_GE.GEORGIAN-ACADEMY', 'ka_GE.GEORGIAN-ACADEMY')
check('cs_CZ.iso88592', 'cs_CZ.ISO8859-2')
check('cs_CZ.ISO8859-2', 'cs_CZ.ISO8859-2')
print("NormalizeTest::test_hyphenated_encoding: ok")
