# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "sum_special_values__test_decimal_inf"
# subject = "cpython.test_statistics.SumSpecialValues.test_decimal_inf"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_statistics
_suite = unittest.defaultTestLoader.loadTestsFromName("SumSpecialValues.test_decimal_inf", test_statistics)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SumSpecialValues.test_decimal_inf did not pass"
print("SumSpecialValues::test_decimal_inf: ok")
