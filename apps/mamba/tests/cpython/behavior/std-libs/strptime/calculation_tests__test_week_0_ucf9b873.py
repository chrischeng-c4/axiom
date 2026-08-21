# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "calculation_tests__test_week_0_ucf9b873"
# subject = "cpython.test_strptime.CalculationTests.test_week_0"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import time
import locale
import re
import os
import platform
import sys
from datetime import date as datetime_date
import _strptime
self_time_tuple = time.gmtime()

def check(value, format, *expected):
    assert _strptime._strptime_time(value, format)[:-1] == expected
check('2015 0 0', '%Y %U %w', 2014, 12, 28, 0, 0, 0, 6, 362)
check('2015 0 0', '%Y %W %w', 2015, 1, 4, 0, 0, 0, 6, 4)
check('2015 1 1', '%G %V %u', 2014, 12, 29, 0, 0, 0, 0, 363)
check('2015 0 1', '%Y %U %w', 2014, 12, 29, 0, 0, 0, 0, 363)
check('2015 0 1', '%Y %W %w', 2014, 12, 29, 0, 0, 0, 0, 363)
check('2015 1 2', '%G %V %u', 2014, 12, 30, 0, 0, 0, 1, 364)
check('2015 0 2', '%Y %U %w', 2014, 12, 30, 0, 0, 0, 1, 364)
check('2015 0 2', '%Y %W %w', 2014, 12, 30, 0, 0, 0, 1, 364)
check('2015 1 3', '%G %V %u', 2014, 12, 31, 0, 0, 0, 2, 365)
check('2015 0 3', '%Y %U %w', 2014, 12, 31, 0, 0, 0, 2, 365)
check('2015 0 3', '%Y %W %w', 2014, 12, 31, 0, 0, 0, 2, 365)
check('2015 1 4', '%G %V %u', 2015, 1, 1, 0, 0, 0, 3, 1)
check('2015 0 4', '%Y %U %w', 2015, 1, 1, 0, 0, 0, 3, 1)
check('2015 0 4', '%Y %W %w', 2015, 1, 1, 0, 0, 0, 3, 1)
check('2015 1 5', '%G %V %u', 2015, 1, 2, 0, 0, 0, 4, 2)
check('2015 0 5', '%Y %U %w', 2015, 1, 2, 0, 0, 0, 4, 2)
check('2015 0 5', '%Y %W %w', 2015, 1, 2, 0, 0, 0, 4, 2)
check('2015 1 6', '%G %V %u', 2015, 1, 3, 0, 0, 0, 5, 3)
check('2015 0 6', '%Y %U %w', 2015, 1, 3, 0, 0, 0, 5, 3)
check('2015 0 6', '%Y %W %w', 2015, 1, 3, 0, 0, 0, 5, 3)
check('2015 1 7', '%G %V %u', 2015, 1, 4, 0, 0, 0, 6, 4)
check('2009 0 0', '%Y %U %w', 2008, 12, 28, 0, 0, 0, 6, 363)
check('2009 0 0', '%Y %W %w', 2009, 1, 4, 0, 0, 0, 6, 4)
check('2009 1 1', '%G %V %u', 2008, 12, 29, 0, 0, 0, 0, 364)
check('2009 0 1', '%Y %U %w', 2008, 12, 29, 0, 0, 0, 0, 364)
check('2009 0 1', '%Y %W %w', 2008, 12, 29, 0, 0, 0, 0, 364)
check('2009 1 2', '%G %V %u', 2008, 12, 30, 0, 0, 0, 1, 365)
check('2009 0 2', '%Y %U %w', 2008, 12, 30, 0, 0, 0, 1, 365)
check('2009 0 2', '%Y %W %w', 2008, 12, 30, 0, 0, 0, 1, 365)
check('2009 1 3', '%G %V %u', 2008, 12, 31, 0, 0, 0, 2, 366)
check('2009 0 3', '%Y %U %w', 2008, 12, 31, 0, 0, 0, 2, 366)
check('2009 0 3', '%Y %W %w', 2008, 12, 31, 0, 0, 0, 2, 366)
check('2009 1 4', '%G %V %u', 2009, 1, 1, 0, 0, 0, 3, 1)
check('2009 0 4', '%Y %U %w', 2009, 1, 1, 0, 0, 0, 3, 1)
check('2009 0 4', '%Y %W %w', 2009, 1, 1, 0, 0, 0, 3, 1)
check('2009 1 5', '%G %V %u', 2009, 1, 2, 0, 0, 0, 4, 2)
check('2009 0 5', '%Y %U %w', 2009, 1, 2, 0, 0, 0, 4, 2)
check('2009 0 5', '%Y %W %w', 2009, 1, 2, 0, 0, 0, 4, 2)
check('2009 1 6', '%G %V %u', 2009, 1, 3, 0, 0, 0, 5, 3)
check('2009 0 6', '%Y %U %w', 2009, 1, 3, 0, 0, 0, 5, 3)
check('2009 0 6', '%Y %W %w', 2009, 1, 3, 0, 0, 0, 5, 3)
check('2009 1 7', '%G %V %u', 2009, 1, 4, 0, 0, 0, 6, 4)

print("CalculationTests::test_week_0: ok")
