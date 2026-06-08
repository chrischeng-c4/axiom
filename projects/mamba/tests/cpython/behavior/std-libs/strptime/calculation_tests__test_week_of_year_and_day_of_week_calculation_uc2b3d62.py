# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "calculation_tests__test_week_of_year_and_day_of_week_calculation_uc2b3d62"
# subject = "cpython.test_strptime.CalculationTests.test_week_of_year_and_day_of_week_calculation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_strptime
_suite = unittest.defaultTestLoader.loadTestsFromName("CalculationTests.test_week_of_year_and_day_of_week_calculation", test_strptime)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CalculationTests.test_week_of_year_and_day_of_week_calculation did not pass"
print("CalculationTests::test_week_of_year_and_day_of_week_calculation: ok")
