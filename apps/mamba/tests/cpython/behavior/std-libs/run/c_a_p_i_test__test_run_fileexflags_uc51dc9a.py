# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "run"
# dimension = "behavior"
# case = "c_a_p_i_test__test_run_fileexflags_uc51dc9a"
# subject = "cpython.test_run.CAPITest.test_run_fileexflags"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_run.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_run
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_run_fileexflags", test_run)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_run_fileexflags did not pass"
print("CAPITest::test_run_fileexflags: ok")
