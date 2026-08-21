# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runpy"
# dimension = "behavior"
# case = "test_exit__test_pymain_run_file_uc7987aa"
# subject = "cpython.test_runpy.TestExit.test_pymain_run_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_runpy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_runpy
_suite = unittest.defaultTestLoader.loadTestsFromName("TestExit.test_pymain_run_file", test_runpy)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestExit.test_pymain_run_file did not pass"
print("TestExit::test_pymain_run_file: ok")
