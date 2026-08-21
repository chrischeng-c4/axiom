# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_bytes__test_byte_filenames_ucff8840"
# subject = "cpython.test_difflib.TestBytes.test_byte_filenames"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_difflib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBytes.test_byte_filenames", test_difflib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBytes.test_byte_filenames did not pass"
print("TestBytes::test_byte_filenames: ok")
