# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "implementation_test__test_deprecated_modules_ucd9d166"
# subject = "cpython.test_re.ImplementationTest.test_deprecated_modules"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_re
_suite = unittest.defaultTestLoader.loadTestsFromName("ImplementationTest.test_deprecated_modules", test_re)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ImplementationTest.test_deprecated_modules did not pass"
print("ImplementationTest::test_deprecated_modules: ok")
