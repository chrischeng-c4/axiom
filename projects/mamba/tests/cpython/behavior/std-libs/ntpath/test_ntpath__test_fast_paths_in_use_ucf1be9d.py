# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "behavior"
# case = "test_ntpath__test_fast_paths_in_use_ucf1be9d"
# subject = "cpython.test_ntpath.TestNtpath.test_fast_paths_in_use"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ntpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ntpath
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNtpath.test_fast_paths_in_use", test_ntpath)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNtpath.test_fast_paths_in_use did not pass"
print("TestNtpath::test_fast_paths_in_use: ok")
