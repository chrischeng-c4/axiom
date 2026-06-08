# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "misc_test__test_useful_error_message_when_modules_missing"
# subject = "cpython.test_tarfile.MiscTest.test_useful_error_message_when_modules_missing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTest.test_useful_error_message_when_modules_missing", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTest.test_useful_error_message_when_modules_missing did not pass"
print("MiscTest::test_useful_error_message_when_modules_missing: ok")
