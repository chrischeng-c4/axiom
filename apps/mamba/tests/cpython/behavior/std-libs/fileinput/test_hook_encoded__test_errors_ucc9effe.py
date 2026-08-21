# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "behavior"
# case = "test_hook_encoded__test_errors_ucc9effe"
# subject = "cpython.test_fileinput.Test_hook_encoded.test_errors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileinput.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_fileinput
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_hook_encoded.test_errors", test_fileinput)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_hook_encoded.test_errors did not pass"
print("Test_hook_encoded::test_errors: ok")
