# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "test_executable_prepended_zip__test_read_zip64_with_exe_prepended"
# subject = "cpython.test_core.TestExecutablePrependedZip.test_read_zip64_with_exe_prepended"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("TestExecutablePrependedZip.test_read_zip64_with_exe_prepended", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestExecutablePrependedZip.test_read_zip64_with_exe_prepended did not pass"
print("TestExecutablePrependedZip::test_read_zip64_with_exe_prepended: ok")
