# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "strptime_tests__test_bad_offset"
# subject = "cpython.test_strptime.StrptimeTests.test_bad_offset"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::StrptimeTests::test_bad_offset
"""Auto-ported test: StrptimeTests::test_bad_offset (CPython 3.12 oracle)."""


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
try:
    _strptime._strptime('-01:30:30.', '%z')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    _strptime._strptime('-0130:30', '%z')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    _strptime._strptime('-01:30:30.1234567', '%z')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    _strptime._strptime('-01:30:30:123456', '%z')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    _strptime._strptime('-01:3030', '%z')
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import types as _types_aR
    err = _types_aR.SimpleNamespace(exception=_aR_e)

assert 'Inconsistent use of : in -01:3030' == str(err.exception)
print("StrptimeTests::test_bad_offset: ok")
