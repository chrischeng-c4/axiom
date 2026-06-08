# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "namespace_pkgs"
# dimension = "behavior"
# case = "dynamic_path_calculation__test_project3_succeeds_uc98b7be"
# subject = "cpython.test_namespace_pkgs.DynamicPathCalculation.test_project3_succeeds"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_namespace_pkgs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_namespace_pkgs
_suite = unittest.defaultTestLoader.loadTestsFromName("DynamicPathCalculation.test_project3_succeeds", test_namespace_pkgs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DynamicPathCalculation.test_project3_succeeds did not pass"
print("DynamicPathCalculation::test_project3_succeeds: ok")
