# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "strptime_tests__test_caseinsensitive"
# subject = "cpython.test_strptime.StrptimeTests.test_caseinsensitive"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::StrptimeTests::test_caseinsensitive
"""Auto-ported test: StrptimeTests::test_caseinsensitive (CPython 3.12 oracle)."""


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
"""Create testing time tuples."""
self_time_tuple = time.localtime()
strf_output = time.strftime('%B', self_time_tuple)

assert _strptime._strptime_time(strf_output.upper(), '%B')

assert _strptime._strptime_time(strf_output.lower(), '%B')

assert _strptime._strptime_time(strf_output.capitalize(), '%B')
print("StrptimeTests::test_caseinsensitive: ok")
