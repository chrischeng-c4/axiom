# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "cache_tests__test_new_localetime"
# subject = "cpython.test_strptime.CacheTests.test_new_localetime"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::CacheTests::test_new_localetime
"""Auto-ported test: CacheTests::test_new_localetime (CPython 3.12 oracle)."""


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
locale_time_id = _strptime._TimeRE_cache.locale_time
_strptime._TimeRE_cache.locale_time.lang = 'Ni'
_strptime._strptime_time('10', '%d')

assert locale_time_id is not _strptime._TimeRE_cache.locale_time
print("CacheTests::test_new_localetime: ok")
