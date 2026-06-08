# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "legacy_base64_test_case__test_encode_ucbbeb3f"
# subject = "cpython.test_base64.LegacyBase64TestCase.test_encode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_base64
_suite = unittest.defaultTestLoader.loadTestsFromName("LegacyBase64TestCase.test_encode", test_base64)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LegacyBase64TestCase.test_encode did not pass"
print("LegacyBase64TestCase::test_encode: ok")
