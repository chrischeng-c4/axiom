# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "u_t_f8_validator_test__test_invalid_utf8_uc5d688f"
# subject = "cpython.test_source_encoding.UTF8ValidatorTest.test_invalid_utf8"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_source_encoding
_suite = unittest.defaultTestLoader.loadTestsFromName("UTF8ValidatorTest.test_invalid_utf8", test_source_encoding)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UTF8ValidatorTest.test_invalid_utf8 did not pass"
print("UTF8ValidatorTest::test_invalid_utf8: ok")
