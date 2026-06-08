# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlretrieve__http_tests__test_short_content_raises_ContentTooShortError"
# subject = "cpython.test_urllib.urlretrieve_HttpTests.test_short_content_raises_ContentTooShortError"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib
_suite = unittest.defaultTestLoader.loadTestsFromName("urlretrieve_HttpTests.test_short_content_raises_ContentTooShortError", test_urllib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython urlretrieve_HttpTests.test_short_content_raises_ContentTooShortError did not pass"
print("urlretrieve_HttpTests::test_short_content_raises_ContentTooShortError: ok")
