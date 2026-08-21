# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "behavior"
# case = "test_i_pv6_environment__test_af"
# subject = "cpython.test_ftplib.TestIPv6Environment.test_af"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ftplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ftplib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestIPv6Environment.test_af", test_ftplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestIPv6Environment.test_af did not pass"
print("TestIPv6Environment::test_af: ok")
