# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "util"
# dimension = "behavior"
# case = "magic_number_tests__test_magic_number"
# subject = "cpython.test_util.MagicNumberTests.test_magic_number"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_util.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_util
_suite = unittest.defaultTestLoader.loadTestsFromName("MagicNumberTests.test_magic_number", test_util)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MagicNumberTests.test_magic_number did not pass"
print("MagicNumberTests::test_magic_number: ok")
