# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "test_pytime__test_localtime_timezone_uc1d6b49"
# subject = "cpython.test_time.TestPytime.test_localtime_timezone"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_time
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPytime.test_localtime_timezone", test_time)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPytime.test_localtime_timezone did not pass"
print("TestPytime::test_localtime_timezone: ok")
