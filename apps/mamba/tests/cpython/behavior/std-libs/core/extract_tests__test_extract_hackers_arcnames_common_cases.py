# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "extract_tests__test_extract_hackers_arcnames_common_cases"
# subject = "cpython.test_core.ExtractTests.test_extract_hackers_arcnames_common_cases"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("ExtractTests.test_extract_hackers_arcnames_common_cases", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExtractTests.test_extract_hackers_arcnames_common_cases did not pass"
print("ExtractTests::test_extract_hackers_arcnames_common_cases: ok")
