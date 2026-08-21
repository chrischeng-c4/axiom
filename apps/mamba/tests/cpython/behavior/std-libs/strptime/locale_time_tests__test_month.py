# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "locale_time_tests__test_month"
# subject = "cpython.test_strptime.LocaleTime_Tests.test_month"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::LocaleTime_Tests::test_month
"""Auto-ported test: LocaleTime_Tests::test_month (CPython 3.12 oracle)."""


import unittest
import time
import locale
import re
import os
import platform
import sys
from test import support
from test.support import skip_if_buggy_ucrt_strfptime, run_with_locales
from datetime import date as datetime_date
import _strptime


'PyUnit testing against strptime'

libc_ver = platform.libc_ver()

if libc_ver[0] == 'glibc':
    glibc_ver = tuple(map(int, libc_ver[1].split('.')))
else:
    glibc_ver = None


# --- test body ---
def compare_against_time(testing, directive, tuple_position, error_msg):
    """Helper method that tests testing against directive based on the
        tuple_position of time_tuple.  Uses error_msg as error message.

        """
    strftime_output = time.strftime(directive, self_time_tuple).lower()
    comparison = testing[self_time_tuple[tuple_position]]

    assert strftime_output in testing

    assert comparison == strftime_output
'Create time tuple based on current time.'
self_time_tuple = time.localtime()
self_LT_ins = _strptime.LocaleTime()
compare_against_time(self_LT_ins.f_month, '%B', 1, 'Testing against full month name failed')
compare_against_time(self_LT_ins.a_month, '%b', 1, 'Testing against abbreviated month name failed')
print("LocaleTime_Tests::test_month: ok")
