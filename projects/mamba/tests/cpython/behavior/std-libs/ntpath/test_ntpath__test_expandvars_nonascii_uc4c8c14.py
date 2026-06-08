# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "behavior"
# case = "test_ntpath__test_expandvars_nonascii_uc4c8c14"
# subject = "cpython.test_ntpath.TestNtpath.test_expandvars_nonascii"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ntpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ntpath
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNtpath.test_expandvars_nonascii", test_ntpath)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNtpath.test_expandvars_nonascii did not pass"
print("TestNtpath::test_expandvars_nonascii: ok")
