# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "normalize_test__test_english"
# subject = "cpython.test_locale.NormalizeTest.test_english"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locale.py::NormalizeTest::test_english
"""Auto-ported test: NormalizeTest::test_english (CPython 3.12 oracle)."""


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
check('en', 'en_US.ISO8859-1')
check('EN', 'en_US.ISO8859-1')
check('en.iso88591', 'en_US.ISO8859-1')
check('en_US', 'en_US.ISO8859-1')
check('en_us', 'en_US.ISO8859-1')
check('en_GB', 'en_GB.ISO8859-1')
check('en_US.UTF-8', 'en_US.UTF-8')
check('en_US.utf8', 'en_US.UTF-8')
check('en_US:UTF-8', 'en_US.UTF-8')
check('en_US.ISO8859-1', 'en_US.ISO8859-1')
check('en_US.US-ASCII', 'en_US.ISO8859-1')
check('en_US.88591', 'en_US.ISO8859-1')
check('en_US.885915', 'en_US.ISO8859-15')
check('english', 'en_EN.ISO8859-1')
check('english_uk.ascii', 'en_GB.ISO8859-1')
print("NormalizeTest::test_english: ok")
