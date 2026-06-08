# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sysconfig"
# dimension = "behavior"
# case = "deprecation_tests__test_is_python_build_check_home_uc1db2cd"
# subject = "cpython.test_sysconfig.DeprecationTests.test_is_python_build_check_home"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sysconfig.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sysconfig
_suite = unittest.defaultTestLoader.loadTestsFromName("DeprecationTests.test_is_python_build_check_home", test_sysconfig)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DeprecationTests.test_is_python_build_check_home did not pass"
print("DeprecationTests::test_is_python_build_check_home: ok")
