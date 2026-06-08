# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_mkstemp__test_for_tempdir_is_bytes_issue40701_api_warts"
# subject = "cpython.test_tempfile.TestMkstemp.test_for_tempdir_is_bytes_issue40701_api_warts"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMkstemp.test_for_tempdir_is_bytes_issue40701_api_warts", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMkstemp.test_for_tempdir_is_bytes_issue40701_api_warts did not pass"
print("TestMkstemp::test_for_tempdir_is_bytes_issue40701_api_warts: ok")
