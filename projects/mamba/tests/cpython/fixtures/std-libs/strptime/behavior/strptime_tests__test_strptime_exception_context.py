# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "strptime_tests__test_strptime_exception_context"
# subject = "cpython.test_strptime.StrptimeTests.test_strptime_exception_context"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::StrptimeTests::test_strptime_exception_context
"""Auto-ported test: StrptimeTests::test_strptime_exception_context (CPython 3.12 oracle)."""


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
    _strptime._strptime_time('', '%D')
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import types as _types_aR
    e = _types_aR.SimpleNamespace(exception=_aR_e)

assert e.exception.__suppress_context__ is True
try:
    _strptime._strptime_time('19', '%Y %')
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import types as _types_aR
    e = _types_aR.SimpleNamespace(exception=_aR_e)

assert e.exception.__context__ is None
print("StrptimeTests::test_strptime_exception_context: ok")
