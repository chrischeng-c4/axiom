# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "test_fr_fr_number_formatting__test_grouping_and_padding"
# subject = "cpython.test_locale.TestFrFRNumberFormatting.test_grouping_and_padding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locale.py::TestFrFRNumberFormatting::test_grouping_and_padding
"""Auto-ported test: TestFrFRNumberFormatting::test_grouping_and_padding (CPython 3.12 oracle)."""


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
cooked_values = {'currency_symbol': '€', 'decimal_point': ',', 'frac_digits': 2, 'grouping': [3, 3, 0], 'int_curr_symbol': 'EUR ', 'int_frac_digits': 2, 'mon_decimal_point': ',', 'mon_grouping': [3, 3, 0], 'mon_thousands_sep': ' ', 'n_cs_precedes': 0, 'n_sep_by_space': 1, 'n_sign_posn': 1, 'negative_sign': '-', 'p_cs_precedes': 0, 'p_sep_by_space': 1, 'p_sign_posn': 1, 'positive_sign': '', 'thousands_sep': ' '}

def _test_currency(value, out, **format_opts):

    assert locale.currency(value, **format_opts) == out

def _test_format_string(format, value, out, **format_opts):

    assert locale.format_string(format, value, **format_opts) == out
locale._override_localeconv = cooked_values
_test_format_string('%6.2f', 345.67, grouping=True, out='345,67')
_test_format_string('%7.2f', 345.67, grouping=True, out=' 345,67')
_test_format_string('%8.2f', 12345.67, grouping=True, out='12 345,67')
_test_format_string('%9.2f', 12345.67, grouping=True, out='12 345,67')
_test_format_string('%10.2f', 12345.67, grouping=True, out=' 12 345,67')
_test_format_string('%-6.2f', 345.67, grouping=True, out='345,67')
_test_format_string('%-7.2f', 345.67, grouping=True, out='345,67 ')
_test_format_string('%-8.2f', 12345.67, grouping=True, out='12 345,67')
_test_format_string('%-9.2f', 12345.67, grouping=True, out='12 345,67')
_test_format_string('%-10.2f', 12345.67, grouping=True, out='12 345,67 ')
print("TestFrFRNumberFormatting::test_grouping_and_padding: ok")
