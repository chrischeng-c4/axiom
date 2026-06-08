# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "api"
# dimension = "behavior"
# case = "frozen_importlib_tests__test_no_frozen_importlib"
# subject = "cpython.test_api.FrozenImportlibTests.test_no_frozen_importlib"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_api.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_api
_suite = unittest.defaultTestLoader.loadTestsFromName("FrozenImportlibTests.test_no_frozen_importlib", test_api)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FrozenImportlibTests.test_no_frozen_importlib did not pass"
print("FrozenImportlibTests::test_no_frozen_importlib: ok")
