# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utils"
# dimension = "behavior"
# case = "date_time_tests__test_aware_datetime_ucc47c8b"
# subject = "cpython.test_utils.DateTimeTests.test_aware_datetime"
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
assert utils.format_datetime(aware_dt) == datestring + offsetstring

print("DateTimeTests::test_aware_datetime: ok")
