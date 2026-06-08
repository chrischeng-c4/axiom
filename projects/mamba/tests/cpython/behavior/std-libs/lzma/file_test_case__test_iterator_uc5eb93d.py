# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "file_test_case__test_iterator_uc5eb93d"
# subject = "cpython.test_lzma.FileTestCase.test_iterator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_lzma
_suite = unittest.defaultTestLoader.loadTestsFromName("FileTestCase.test_iterator", test_lzma)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FileTestCase.test_iterator did not pass"
print("FileTestCase::test_iterator: ok")
