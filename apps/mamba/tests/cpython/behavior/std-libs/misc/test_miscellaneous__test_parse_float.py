# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "test_miscellaneous__test_parse_float"
# subject = "cpython.test_misc.TestMiscellaneous.test_parse_float"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_tomllib import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMiscellaneous.test_parse_float", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMiscellaneous.test_parse_float did not pass"
print("TestMiscellaneous::test_parse_float: ok")
