# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "tests_with_multiple_opens__test_write_while_reading"
# subject = "cpython.test_core.TestsWithMultipleOpens.test_write_while_reading"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("TestsWithMultipleOpens.test_write_while_reading", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestsWithMultipleOpens.test_write_while_reading did not pass"
print("TestsWithMultipleOpens::test_write_while_reading: ok")
