# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "locale_time_tests__test_am_pm"
# subject = "cpython.test_strptime.LocaleTime_Tests.test_am_pm"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::LocaleTime_Tests::test_am_pm
"""Auto-ported test: LocaleTime_Tests::test_am_pm (CPython 3.12 oracle)."""


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
strftime_output = time.strftime('%p', self_time_tuple).lower()

assert strftime_output in self_LT_ins.am_pm
if self_time_tuple[3] < 12:
    position = 0
else:
    position = 1

assert self_LT_ins.am_pm[position] == strftime_output
print("LocaleTime_Tests::test_am_pm: ok")
