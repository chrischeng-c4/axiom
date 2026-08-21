# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "import"
# dimension = "behavior"
# case = "import_tests__test_importmoduleex_uc435e24"
# subject = "cpython.test_import.ImportTests.test_importmoduleex"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_import.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_import
_suite = unittest.defaultTestLoader.loadTestsFromName("ImportTests.test_importmoduleex", test_import)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ImportTests.test_importmoduleex did not pass"
print("ImportTests::test_importmoduleex: ok")
