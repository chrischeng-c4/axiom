# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lazy"
# dimension = "behavior"
# case = "lazy_loader_factory_tests__test_init_uc926f04"
# subject = "cpython.test_lazy.LazyLoaderFactoryTests.test_init"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_lazy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_lazy
_suite = unittest.defaultTestLoader.loadTestsFromName("LazyLoaderFactoryTests.test_init", test_lazy)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LazyLoaderFactoryTests.test_init did not pass"
print("LazyLoaderFactoryTests::test_init: ok")
