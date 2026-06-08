# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "test_miscellaneous__test_getencoding"
# subject = "cpython.test_locale.TestMiscellaneous.test_getencoding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locale.py::TestMiscellaneous::test_getencoding
"""Auto-ported test: TestMiscellaneous::test_getencoding (CPython 3.12 oracle)."""


from decimal import Decimal
from test.support import verbose, is_android, is_emscripten, is_wasi
from test.support.warnings_helper import check_warnings
import unittest
import locale
import sys
import codecs


class BaseLocalizedTest(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        if sys.platform == 'darwin':
            import os
            tlocs = ('en_US.UTF-8', 'en_US.ISO8859-1', 'en_US')
            if int(os.uname().release.split('.')[0]) < 10:
                raise unittest.SkipTest('Locale support on MacOSX is minimal')
        elif sys.platform.startswith('win'):
            tlocs = ('En', 'English')
        else:
            tlocs = ('en_US.UTF-8', 'en_US.ISO8859-1', 'en_US.US-ASCII', 'en_US')
        try:
            oldlocale = locale.setlocale(locale.LC_NUMERIC)
            for tloc in tlocs:
                try:
                    locale.setlocale(locale.LC_NUMERIC, tloc)
                except locale.Error:
                    continue
                break
            else:
                raise unittest.SkipTest('Test locale not supported (tried %s)' % ', '.join(tlocs))
            cls.enUS_locale = tloc
        finally:
            locale.setlocale(locale.LC_NUMERIC, oldlocale)

    def setUp(self):
        oldlocale = locale.setlocale(self.locale_type)
        self.addCleanup(locale.setlocale, self.locale_type, oldlocale)
        locale.setlocale(self.locale_type, self.enUS_locale)
        if verbose:
            print('testing with %r...' % self.enUS_locale, end=' ', flush=True)

class BaseCookedTest(unittest.TestCase):

    def setUp(self):
        locale._override_localeconv = self.cooked_values

    def tearDown(self):
        locale._override_localeconv = {}

class CCookedTest(BaseCookedTest):
    cooked_values = {'currency_symbol': '', 'decimal_point': '.', 'frac_digits': 127, 'grouping': [], 'int_curr_symbol': '', 'int_frac_digits': 127, 'mon_decimal_point': '', 'mon_grouping': [], 'mon_thousands_sep': '', 'n_cs_precedes': 127, 'n_sep_by_space': 127, 'n_sign_posn': 127, 'negative_sign': '', 'p_cs_precedes': 127, 'p_sep_by_space': 127, 'p_sign_posn': 127, 'positive_sign': '', 'thousands_sep': ''}

class EnUSCookedTest(BaseCookedTest):
    cooked_values = {'currency_symbol': '$', 'decimal_point': '.', 'frac_digits': 2, 'grouping': [3, 3, 0], 'int_curr_symbol': 'USD ', 'int_frac_digits': 2, 'mon_decimal_point': '.', 'mon_grouping': [3, 3, 0], 'mon_thousands_sep': ',', 'n_cs_precedes': 1, 'n_sep_by_space': 0, 'n_sign_posn': 1, 'negative_sign': '-', 'p_cs_precedes': 1, 'p_sep_by_space': 0, 'p_sign_posn': 1, 'positive_sign': '', 'thousands_sep': ','}

class FrFRCookedTest(BaseCookedTest):
    cooked_values = {'currency_symbol': '€', 'decimal_point': ',', 'frac_digits': 2, 'grouping': [3, 3, 0], 'int_curr_symbol': 'EUR ', 'int_frac_digits': 2, 'mon_decimal_point': ',', 'mon_grouping': [3, 3, 0], 'mon_thousands_sep': ' ', 'n_cs_precedes': 0, 'n_sep_by_space': 1, 'n_sign_posn': 1, 'negative_sign': '-', 'p_cs_precedes': 0, 'p_sep_by_space': 1, 'p_sign_posn': 1, 'positive_sign': '', 'thousands_sep': ' '}

class BaseFormattingTest(object):

    def _test_format_string(self, format, value, out, **format_opts):
        self.assertEqual(locale.format_string(format, value, **format_opts), out)

    def _test_currency(self, value, out, **format_opts):
        self.assertEqual(locale.currency(value, **format_opts), out)

class BaseDelocalizeTest(BaseLocalizedTest):

    def _test_delocalize(self, value, out):
        self.assertEqual(locale.delocalize(value), out)

    def _test_atof(self, value, out):
        self.assertEqual(locale.atof(value), out)

    def _test_atoi(self, value, out):
        self.assertEqual(locale.atoi(value), out)

class BaseLocalizeTest(BaseLocalizedTest):

    def _test_localize(self, value, out, grouping=False):
        self.assertEqual(locale.localize(value, grouping=grouping), out)


# --- test body ---
enc = locale.getencoding()

assert isinstance(enc, str)

assert enc != ''
codecs.lookup(enc)
print("TestMiscellaneous::test_getencoding: ok")
