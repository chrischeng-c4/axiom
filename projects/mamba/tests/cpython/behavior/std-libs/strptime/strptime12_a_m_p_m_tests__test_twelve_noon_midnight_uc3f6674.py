# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "strptime12_a_m_p_m_tests__test_twelve_noon_midnight_uc3f6674"
# subject = "cpython.test_strptime.Strptime12AMPMTests.test_twelve_noon_midnight"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_strptime
_suite = unittest.defaultTestLoader.loadTestsFromName("Strptime12AMPMTests.test_twelve_noon_midnight", test_strptime)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Strptime12AMPMTests.test_twelve_noon_midnight did not pass"
print("Strptime12AMPMTests::test_twelve_noon_midnight: ok")
