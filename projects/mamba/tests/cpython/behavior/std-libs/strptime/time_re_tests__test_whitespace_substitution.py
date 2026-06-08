# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "time_re_tests__test_whitespace_substitution"
# subject = "cpython.test_strptime.TimeRETests.test_whitespace_substitution"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::TimeRETests::test_whitespace_substitution
"""Auto-ported test: TimeRETests::test_whitespace_substitution (CPython 3.12 oracle)."""


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
pattern = self_time_re.pattern('%j %H')

assert not re.match(pattern, '180')

assert re.match(pattern, '18 0')
print("TimeRETests::test_whitespace_substitution: ok")
