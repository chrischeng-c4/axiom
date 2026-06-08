# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlretrieve__http_tests__test_short_content_raises_ContentTooShortError_without_reporthook"
# subject = "cpython.test_urllib.urlretrieve_HttpTests.test_short_content_raises_ContentTooShortError_without_reporthook"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib
_suite = unittest.defaultTestLoader.loadTestsFromName("urlretrieve_HttpTests.test_short_content_raises_ContentTooShortError_without_reporthook", test_urllib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython urlretrieve_HttpTests.test_short_content_raises_ContentTooShortError_without_reporthook did not pass"
print("urlretrieve_HttpTests::test_short_content_raises_ContentTooShortError_without_reporthook: ok")
