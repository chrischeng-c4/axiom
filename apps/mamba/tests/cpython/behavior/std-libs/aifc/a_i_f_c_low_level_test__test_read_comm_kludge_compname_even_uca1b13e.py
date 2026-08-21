# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "behavior"
# case = "a_i_f_c_low_level_test__test_read_comm_kludge_compname_even_uca1b13e"
# subject = "cpython.test_aifc.AIFCLowLevelTest.test_read_comm_kludge_compname_even"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_aifc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_aifc
_suite = unittest.defaultTestLoader.loadTestsFromName("AIFCLowLevelTest.test_read_comm_kludge_compname_even", test_aifc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AIFCLowLevelTest.test_read_comm_kludge_compname_even did not pass"
print("AIFCLowLevelTest::test_read_comm_kludge_compname_even: ok")
