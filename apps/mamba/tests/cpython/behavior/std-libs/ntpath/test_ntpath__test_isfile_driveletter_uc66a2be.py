# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "behavior"
# case = "test_ntpath__test_isfile_driveletter_uc66a2be"
# subject = "cpython.test_ntpath.TestNtpath.test_isfile_driveletter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ntpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ntpath
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNtpath.test_isfile_driveletter", test_ntpath)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNtpath.test_isfile_driveletter did not pass"
print("TestNtpath::test_isfile_driveletter: ok")
