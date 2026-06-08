# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "generator"
# dimension = "behavior"
# case = "test_generator__test_verify_generated_headers_uc986ae8"
# subject = "cpython.test_generator.TestGenerator.test_verify_generated_headers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_generator.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_generator
_suite = unittest.defaultTestLoader.loadTestsFromName("TestGenerator.test_verify_generated_headers", test_generator)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestGenerator.test_verify_generated_headers did not pass"
print("TestGenerator::test_verify_generated_headers: ok")
