# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "locale_time_tests__test_date_time"
# subject = "cpython.test_strptime.LocaleTime_Tests.test_date_time"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::LocaleTime_Tests::test_date_time
"""Auto-ported test: LocaleTime_Tests::test_date_time (CPython 3.12 oracle)."""


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
"""Create time tuple based on current time."""
self_time_tuple = time.localtime()
self_LT_ins = _strptime.LocaleTime()
magic_date = (1999, 3, 17, 22, 44, 55, 2, 76, 0)
strftime_output = time.strftime('%c', magic_date)

assert time.strftime(self_LT_ins.LC_date_time, magic_date) == strftime_output
strftime_output = time.strftime('%x', magic_date)

assert time.strftime(self_LT_ins.LC_date, magic_date) == strftime_output
strftime_output = time.strftime('%X', magic_date)

assert time.strftime(self_LT_ins.LC_time, magic_date) == strftime_output
LT = _strptime.LocaleTime()
LT.am_pm = ('', '')

assert LT.LC_time
print("LocaleTime_Tests::test_date_time: ok")
