# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericalias"
# dimension = "behavior"
# case = "base_test__test_repr_uc4023c6"
# subject = "cpython.test_genericalias.BaseTest.test_repr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericalias.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_genericalias
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseTest.test_repr", test_genericalias)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseTest.test_repr did not pass"
print("BaseTest::test_repr: ok")
