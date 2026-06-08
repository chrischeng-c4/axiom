# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "htmlparser"
# dimension = "behavior"
# case = "test_inheritance__test_base_class_methods_called_uc798b2f"
# subject = "cpython.test_htmlparser.TestInheritance.test_base_class_methods_called"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_htmlparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_htmlparser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInheritance.test_base_class_methods_called", test_htmlparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInheritance.test_base_class_methods_called did not pass"
print("TestInheritance::test_base_class_methods_called: ok")
