# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "strptime_tests__test_second"
# subject = "cpython.test_strptime.StrptimeTests.test_second"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::StrptimeTests::test_second
"""Auto-ported test: StrptimeTests::test_second (CPython 3.12 oracle)."""


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
def roundtrip(fmt, position, time_tuple=None):
    """Helper fxn in testing."""
    if time_tuple is None:
        time_tuple = self_time_tuple
    strf_output = time.strftime(fmt, time_tuple)
    strp_output = _strptime._strptime_time(strf_output, fmt)

    assert strp_output[position] == time_tuple[position]
    if support.verbose >= 3:
        print('testing of %r format: %r -> %r' % (fmt, strf_output, strp_output[position]))
'Create testing time tuples.'
self_time_tuple = time.localtime()
roundtrip('%S', 5)
print("StrptimeTests::test_second: ok")
