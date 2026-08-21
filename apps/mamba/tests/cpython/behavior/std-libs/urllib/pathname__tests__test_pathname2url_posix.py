# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "pathname__tests__test_pathname2url_posix"
# subject = "cpython.test_urllib.Pathname_Tests.test_pathname2url_posix"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib
_suite = unittest.defaultTestLoader.loadTestsFromName("Pathname_Tests.test_pathname2url_posix", test_urllib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Pathname_Tests.test_pathname2url_posix did not pass"
print("Pathname_Tests::test_pathname2url_posix: ok")
