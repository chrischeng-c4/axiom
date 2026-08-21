# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dump"
# dimension = "behavior"
# case = "test_c_dump__test_large_list_ucaff6bf"
# subject = "cpython.test_dump.TestCDump.test_large_list"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/test_dump.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_json import test_dump
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCDump.test_large_list", test_dump)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCDump.test_large_list did not pass"
print("TestCDump::test_large_list: ok")
