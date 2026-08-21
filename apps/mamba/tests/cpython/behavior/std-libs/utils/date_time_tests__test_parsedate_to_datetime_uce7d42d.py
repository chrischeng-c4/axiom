# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utils"
# dimension = "behavior"
# case = "date_time_tests__test_parsedate_to_datetime_uce7d42d"
# subject = "cpython.test_utils.DateTimeTests.test_parsedate_to_datetime"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_utils.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import datetime
from email import utils
import time
import sys
import os.path
import zoneinfo
datestring = 'Sun, 23 Sep 2001 20:10:55'
dateargs = (2001, 9, 23, 20, 10, 55)
offsetstring = ' -0700'
utcoffset = datetime.timedelta(hours=-7)
tz = datetime.timezone(utcoffset)
naive_dt = datetime.datetime(*dateargs)
aware_dt = datetime.datetime(*dateargs, tzinfo=tz)
assert utils.parsedate_to_datetime(datestring + offsetstring) == aware_dt

print("DateTimeTests::test_parsedate_to_datetime: ok")
