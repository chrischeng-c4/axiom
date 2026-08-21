# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "other_tests__test_closed_zip_raises_ValueError"
# subject = "cpython.test_core.OtherTests.test_closed_zip_raises_ValueError"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("OtherTests.test_closed_zip_raises_ValueError", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OtherTests.test_closed_zip_raises_ValueError did not pass"
print("OtherTests::test_closed_zip_raises_ValueError: ok")
