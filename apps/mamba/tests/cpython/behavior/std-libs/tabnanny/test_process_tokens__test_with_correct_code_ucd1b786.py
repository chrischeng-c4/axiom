# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tabnanny"
# dimension = "behavior"
# case = "test_process_tokens__test_with_correct_code_ucd1b786"
# subject = "cpython.test_tabnanny.TestProcessTokens.test_with_correct_code"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tabnanny.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tabnanny
_suite = unittest.defaultTestLoader.loadTestsFromName("TestProcessTokens.test_with_correct_code", test_tabnanny)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestProcessTokens.test_with_correct_code did not pass"
print("TestProcessTokens::test_with_correct_code: ok")
