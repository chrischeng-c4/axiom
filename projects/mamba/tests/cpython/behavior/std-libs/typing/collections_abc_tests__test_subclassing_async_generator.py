# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "collections_abc_tests__test_subclassing_async_generator"
# subject = "cpython.test_typing.CollectionsAbcTests.test_subclassing_async_generator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("CollectionsAbcTests.test_subclassing_async_generator", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CollectionsAbcTests.test_subclassing_async_generator did not pass"
print("CollectionsAbcTests::test_subclassing_async_generator: ok")
