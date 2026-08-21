# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "normalize_test__test_euro_modifier"
# subject = "cpython.test_locale.NormalizeTest.test_euro_modifier"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locale.py::NormalizeTest::test_euro_modifier
"""Auto-ported test: NormalizeTest::test_euro_modifier (CPython 3.12 oracle)."""


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
check('de_DE@euro', 'de_DE.ISO8859-15')
check('en_US.ISO8859-15@euro', 'en_US.ISO8859-15')
check('de_DE.utf8@euro', 'de_DE.UTF-8')
print("NormalizeTest::test_euro_modifier: ok")
