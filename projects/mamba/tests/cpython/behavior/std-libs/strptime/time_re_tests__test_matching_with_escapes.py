# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "time_re_tests__test_matching_with_escapes"
# subject = "cpython.test_strptime.TimeRETests.test_matching_with_escapes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::TimeRETests::test_matching_with_escapes
"""Auto-ported test: TimeRETests::test_matching_with_escapes (CPython 3.12 oracle)."""


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
compiled_re = self_time_re.compile('\\w+ %m')
found = compiled_re.match('\\w+ 10')

assert found
print("TimeRETests::test_matching_with_escapes: ok")
