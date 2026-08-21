# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "behavior"
# case = "read_tests__test_read_all_uc03ef27"
# subject = "cpython.test_telnetlib.ReadTests.test_read_all"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_telnetlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_telnetlib
_suite = unittest.defaultTestLoader.loadTestsFromName("ReadTests.test_read_all", test_telnetlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReadTests.test_read_all did not pass"
print("ReadTests::test_read_all: ok")
