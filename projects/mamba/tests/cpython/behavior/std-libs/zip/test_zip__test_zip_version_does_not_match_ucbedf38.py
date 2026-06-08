# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zip"
# dimension = "behavior"
# case = "test_zip__test_zip_version_does_not_match_ucbedf38"
# subject = "cpython.test_zip.TestZip.test_zip_version_does_not_match"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_zip.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_zip
_suite = unittest.defaultTestLoader.loadTestsFromName("TestZip.test_zip_version_does_not_match", test_zip)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestZip.test_zip_version_does_not_match did not pass"
print("TestZip::test_zip_version_does_not_match: ok")
