# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "test_en_u_s_delocalize__test_atoi_uc3b9168"
# subject = "cpython.test_locale.TestEnUSDelocalize.test_atoi"
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
cooked_values = {'currency_symbol': '$', 'decimal_point': '.', 'frac_digits': 2, 'grouping': [3, 3, 0], 'int_curr_symbol': 'USD ', 'int_frac_digits': 2, 'mon_decimal_point': '.', 'mon_grouping': [3, 3, 0], 'mon_thousands_sep': ',', 'n_cs_precedes': 1, 'n_sep_by_space': 0, 'n_sign_posn': 1, 'negative_sign': '-', 'p_cs_precedes': 1, 'p_sep_by_space': 0, 'p_sign_posn': 1, 'positive_sign': '', 'thousands_sep': ','}

def _test_delocalize(value, out):
    assert locale.delocalize(value) == out

def _test_atof(value, out):
    assert locale.atof(value) == out

def _test_atoi(value, out):
    assert locale.atoi(value) == out
locale._override_localeconv = cooked_values
_test_atoi('50000', 50000)
_test_atoi('50,000', 50000)

print("TestEnUSDelocalize::test_atoi: ok")
