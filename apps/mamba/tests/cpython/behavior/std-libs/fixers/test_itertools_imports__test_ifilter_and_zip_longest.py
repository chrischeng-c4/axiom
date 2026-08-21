# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fixers"
# dimension = "behavior"
# case = "test_itertools_imports__test_ifilter_and_zip_longest"
# subject = "cpython.test_fixers.Test_itertools_imports.test_ifilter_and_zip_longest"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_fixers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_fixers
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_itertools_imports.test_ifilter_and_zip_longest", test_fixers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_itertools_imports.test_ifilter_and_zip_longest did not pass"
print("Test_itertools_imports::test_ifilter_and_zip_longest: ok")
