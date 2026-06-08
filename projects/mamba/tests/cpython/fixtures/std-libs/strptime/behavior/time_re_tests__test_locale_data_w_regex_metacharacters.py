# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "time_re_tests__test_locale_data_w_regex_metacharacters"
# subject = "cpython.test_strptime.TimeRETests.test_locale_data_w_regex_metacharacters"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::TimeRETests::test_locale_data_w_regex_metacharacters
"""Auto-ported test: TimeRETests::test_locale_data_w_regex_metacharacters (CPython 3.12 oracle)."""


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
"""Construct generic TimeRE object."""
self_time_re = _strptime.TimeRE()
self_locale_time = _strptime.LocaleTime()
locale_time = _strptime.LocaleTime()
locale_time.timezone = (frozenset(('utc', 'gmt', 'Tokyo (standard time)')), frozenset('Tokyo (daylight time)'))
time_re = _strptime.TimeRE(locale_time)

assert time_re.compile('%Z').match('Tokyo (standard time)')
print("TimeRETests::test_locale_data_w_regex_metacharacters: ok")
