# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "traceback_cases__test_encoded_file"
# subject = "cpython.test_traceback.TracebackCases.test_encoded_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("TracebackCases.test_encoded_file", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TracebackCases.test_encoded_file did not pass"
print("TracebackCases::test_encoded_file: ok")
