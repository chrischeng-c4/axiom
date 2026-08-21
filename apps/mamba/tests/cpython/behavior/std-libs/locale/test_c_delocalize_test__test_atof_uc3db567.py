# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "test_c_delocalize_test__test_atof_uc3db567"
# subject = "cpython.test_locale.TestCDelocalizeTest.test_atof"
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
cooked_values = {'currency_symbol': '', 'decimal_point': '.', 'frac_digits': 127, 'grouping': [], 'int_curr_symbol': '', 'int_frac_digits': 127, 'mon_decimal_point': '', 'mon_grouping': [], 'mon_thousands_sep': '', 'n_cs_precedes': 127, 'n_sep_by_space': 127, 'n_sign_posn': 127, 'negative_sign': '', 'p_cs_precedes': 127, 'p_sep_by_space': 127, 'p_sign_posn': 127, 'positive_sign': '', 'thousands_sep': ''}

def _test_delocalize(value, out):
    assert locale.delocalize(value) == out

def _test_atof(value, out):
    assert locale.atof(value) == out

def _test_atoi(value, out):
    assert locale.atoi(value) == out
locale._override_localeconv = cooked_values
_test_atof('50000.00', 50000.0)

print("TestCDelocalizeTest::test_atof: ok")
