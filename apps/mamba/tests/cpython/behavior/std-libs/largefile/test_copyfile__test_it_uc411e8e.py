# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "largefile"
# dimension = "behavior"
# case = "test_copyfile__test_it_uc411e8e"
# subject = "cpython.test_largefile.TestCopyfile.test_it"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_largefile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_largefile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCopyfile.test_it", test_largefile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCopyfile.test_it did not pass"
print("TestCopyfile::test_it: ok")
