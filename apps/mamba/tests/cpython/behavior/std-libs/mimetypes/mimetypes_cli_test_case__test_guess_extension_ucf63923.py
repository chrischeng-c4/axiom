# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mimetypes_cli_test_case__test_guess_extension_ucf63923"
# subject = "cpython.test_mimetypes.MimetypesCliTestCase.test_guess_extension"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mimetypes
_suite = unittest.defaultTestLoader.loadTestsFromName("MimetypesCliTestCase.test_guess_extension", test_mimetypes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MimetypesCliTestCase.test_guess_extension did not pass"
print("MimetypesCliTestCase::test_guess_extension: ok")
