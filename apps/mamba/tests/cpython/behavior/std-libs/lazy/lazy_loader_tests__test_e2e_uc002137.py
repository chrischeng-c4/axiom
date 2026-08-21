# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lazy"
# dimension = "behavior"
# case = "lazy_loader_tests__test_e2e_uc002137"
# subject = "cpython.test_lazy.LazyLoaderTests.test_e2e"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_lazy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_lazy
_suite = unittest.defaultTestLoader.loadTestsFromName("LazyLoaderTests.test_e2e", test_lazy)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LazyLoaderTests.test_e2e did not pass"
print("LazyLoaderTests::test_e2e: ok")
