# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "c_functionality__test_c_ieee_context"
# subject = "cpython.test_decimal.CFunctionality.test_c_ieee_context"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_decimal
_suite = unittest.defaultTestLoader.loadTestsFromName("CFunctionality.test_c_ieee_context", test_decimal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CFunctionality.test_c_ieee_context did not pass"
print("CFunctionality::test_c_ieee_context: ok")
