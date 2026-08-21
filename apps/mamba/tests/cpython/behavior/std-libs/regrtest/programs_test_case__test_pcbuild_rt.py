# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regrtest"
# dimension = "behavior"
# case = "programs_test_case__test_pcbuild_rt"
# subject = "cpython.test_regrtest.ProgramsTestCase.test_pcbuild_rt"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_regrtest.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_regrtest
_suite = unittest.defaultTestLoader.loadTestsFromName("ProgramsTestCase.test_pcbuild_rt", test_regrtest)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProgramsTestCase.test_pcbuild_rt did not pass"
print("ProgramsTestCase::test_pcbuild_rt: ok")
