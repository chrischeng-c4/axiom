# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlretrieve__file_tests__test_reporthook"
# subject = "cpython.test_urllib.urlretrieve_FileTests.test_reporthook"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib
_suite = unittest.defaultTestLoader.loadTestsFromName("urlretrieve_FileTests.test_reporthook", test_urllib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython urlretrieve_FileTests.test_reporthook did not pass"
print("urlretrieve_FileTests::test_reporthook: ok")
