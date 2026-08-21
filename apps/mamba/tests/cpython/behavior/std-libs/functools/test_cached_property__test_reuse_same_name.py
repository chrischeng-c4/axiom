# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "test_cached_property__test_reuse_same_name"
# subject = "cpython.test_functools.TestCachedProperty.test_reuse_same_name"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_functools
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCachedProperty.test_reuse_same_name", test_functools)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCachedProperty.test_reuse_same_name did not pass"
print("TestCachedProperty::test_reuse_same_name: ok")
