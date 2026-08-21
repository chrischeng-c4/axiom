# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "c_whitebox__test_c_signal_dict"
# subject = "cpython.test_decimal.CWhitebox.test_c_signal_dict"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_decimal
_suite = unittest.defaultTestLoader.loadTestsFromName("CWhitebox.test_c_signal_dict", test_decimal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CWhitebox.test_c_signal_dict did not pass"
print("CWhitebox::test_c_signal_dict: ok")
