# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "generator"
# dimension = "behavior"
# case = "test_bytes_generator__test_cte_type_7bit_transforms_8bit_cte_ucd4af92"
# subject = "cpython.test_generator.TestBytesGenerator.test_cte_type_7bit_transforms_8bit_cte"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_generator.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_generator
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBytesGenerator.test_cte_type_7bit_transforms_8bit_cte", test_generator)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBytesGenerator.test_cte_type_7bit_transforms_8bit_cte did not pass"
print("TestBytesGenerator::test_cte_type_7bit_transforms_8bit_cte: ok")
