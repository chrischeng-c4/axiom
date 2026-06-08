# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "normalize_test__test_devanagari_modifier"
# subject = "cpython.test_locale.NormalizeTest.test_devanagari_modifier"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locale.py::NormalizeTest::test_devanagari_modifier
"""Auto-ported test: NormalizeTest::test_devanagari_modifier (CPython 3.12 oracle)."""


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
check('ks_IN.UTF-8@devanagari', 'ks_IN.UTF-8@devanagari')
check('ks_IN@devanagari', 'ks_IN.UTF-8@devanagari')
check('ks@devanagari', 'ks_IN.UTF-8@devanagari')
check('ks_IN.UTF-8', 'ks_IN.UTF-8')
check('ks_IN', 'ks_IN.UTF-8')
check('ks', 'ks_IN.UTF-8')
check('sd_IN.UTF-8@devanagari', 'sd_IN.UTF-8@devanagari')
check('sd_IN@devanagari', 'sd_IN.UTF-8@devanagari')
check('sd@devanagari', 'sd_IN.UTF-8@devanagari')
check('sd_IN.UTF-8', 'sd_IN.UTF-8')
check('sd_IN', 'sd_IN.UTF-8')
check('sd', 'sd_IN.UTF-8')
print("NormalizeTest::test_devanagari_modifier: ok")
