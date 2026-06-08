# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "cache_tests__test_time_re_recreation"
# subject = "cpython.test_strptime.CacheTests.test_time_re_recreation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::CacheTests::test_time_re_recreation
"""Auto-ported test: CacheTests::test_time_re_recreation (CPython 3.12 oracle)."""


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
_strptime._strptime_time('10', '%d')
_strptime._strptime_time('2005', '%Y')
_strptime._TimeRE_cache.locale_time.lang = 'Ni'
original_time_re = _strptime._TimeRE_cache
_strptime._strptime_time('10', '%d')

assert original_time_re is not _strptime._TimeRE_cache

assert len(_strptime._regex_cache) == 1
print("CacheTests::test_time_re_recreation: ok")
