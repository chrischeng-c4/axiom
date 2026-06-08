# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlopen__data_tests__test_info"
# subject = "cpython.test_urllib.urlopen_DataTests.test_info"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib
_suite = unittest.defaultTestLoader.loadTestsFromName("urlopen_DataTests.test_info", test_urllib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython urlopen_DataTests.test_info did not pass"
print("urlopen_DataTests::test_info: ok")
