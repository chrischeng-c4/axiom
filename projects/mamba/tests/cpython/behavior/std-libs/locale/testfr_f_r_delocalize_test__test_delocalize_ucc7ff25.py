# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "testfr_f_r_delocalize_test__test_delocalize_ucc7ff25"
# subject = "cpython.test_locale.TestfrFRDelocalizeTest.test_delocalize"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from decimal import Decimal
import locale
import sys
import codecs
cooked_values = {'currency_symbol': '€', 'decimal_point': ',', 'frac_digits': 2, 'grouping': [3, 3, 0], 'int_curr_symbol': 'EUR ', 'int_frac_digits': 2, 'mon_decimal_point': ',', 'mon_grouping': [3, 3, 0], 'mon_thousands_sep': ' ', 'n_cs_precedes': 0, 'n_sep_by_space': 1, 'n_sign_posn': 1, 'negative_sign': '-', 'p_cs_precedes': 0, 'p_sep_by_space': 1, 'p_sign_posn': 1, 'positive_sign': '', 'thousands_sep': ' '}

def _test_delocalize(value, out):
    assert locale.delocalize(value) == out

def _test_atof(value, out):
    assert locale.atof(value) == out

def _test_atoi(value, out):
    assert locale.atoi(value) == out
locale._override_localeconv = cooked_values
_test_delocalize('50000,00', '50000.00')
_test_delocalize('50 000,00', '50000.00')

print("TestfrFRDelocalizeTest::test_delocalize: ok")
