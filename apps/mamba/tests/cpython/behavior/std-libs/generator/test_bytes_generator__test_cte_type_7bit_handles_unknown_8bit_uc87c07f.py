# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "generator"
# dimension = "behavior"
# case = "test_bytes_generator__test_cte_type_7bit_handles_unknown_8bit_uc87c07f"
# subject = "cpython.test_generator.TestBytesGenerator.test_cte_type_7bit_handles_unknown_8bit"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_generator.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_generator
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBytesGenerator.test_cte_type_7bit_handles_unknown_8bit", test_generator)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBytesGenerator.test_cte_type_7bit_handles_unknown_8bit did not pass"
print("TestBytesGenerator::test_cte_type_7bit_handles_unknown_8bit: ok")
