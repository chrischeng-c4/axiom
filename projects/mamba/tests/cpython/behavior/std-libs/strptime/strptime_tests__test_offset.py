# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "strptime_tests__test_offset"
# subject = "cpython.test_strptime.StrptimeTests.test_offset"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::StrptimeTests::test_offset
"""Auto-ported test: StrptimeTests::test_offset (CPython 3.12 oracle)."""


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
one_hour = 60 * 60
half_hour = 30 * 60
half_minute = 30
(*_, offset), _, offset_fraction = _strptime._strptime('+0130', '%z')

assert offset == one_hour + half_hour

assert offset_fraction == 0
(*_, offset), _, offset_fraction = _strptime._strptime('-0100', '%z')

assert offset == -one_hour

assert offset_fraction == 0
(*_, offset), _, offset_fraction = _strptime._strptime('-013030', '%z')

assert offset == -(one_hour + half_hour + half_minute)

assert offset_fraction == 0
(*_, offset), _, offset_fraction = _strptime._strptime('-013030.000001', '%z')

assert offset == -(one_hour + half_hour + half_minute)

assert offset_fraction == -1
(*_, offset), _, offset_fraction = _strptime._strptime('+01:00', '%z')

assert offset == one_hour

assert offset_fraction == 0
(*_, offset), _, offset_fraction = _strptime._strptime('-01:30', '%z')

assert offset == -(one_hour + half_hour)

assert offset_fraction == 0
(*_, offset), _, offset_fraction = _strptime._strptime('-01:30:30', '%z')

assert offset == -(one_hour + half_hour + half_minute)

assert offset_fraction == 0
(*_, offset), _, offset_fraction = _strptime._strptime('-01:30:30.000001', '%z')

assert offset == -(one_hour + half_hour + half_minute)

assert offset_fraction == -1
(*_, offset), _, offset_fraction = _strptime._strptime('+01:30:30.001', '%z')

assert offset == one_hour + half_hour + half_minute

assert offset_fraction == 1000
(*_, offset), _, offset_fraction = _strptime._strptime('Z', '%z')

assert offset == 0

assert offset_fraction == 0
print("StrptimeTests::test_offset: ok")
