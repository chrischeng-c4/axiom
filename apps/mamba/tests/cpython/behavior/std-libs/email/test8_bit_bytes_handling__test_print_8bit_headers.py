# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test8_bit_bytes_handling__test_print_8bit_headers"
# subject = "cpython.test_email.Test8BitBytesHandling.test_print_8bit_headers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("Test8BitBytesHandling.test_print_8bit_headers", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test8BitBytesHandling.test_print_8bit_headers did not pass"
print("Test8BitBytesHandling::test_print_8bit_headers: ok")
