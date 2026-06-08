# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "behavior"
# case = "repr_tests__test_container_ucbbc175"
# subject = "cpython.test_reprlib.ReprTests.test_container"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_reprlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_reprlib
_suite = unittest.defaultTestLoader.loadTestsFromName("ReprTests.test_container", test_reprlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReprTests.test_container did not pass"
print("ReprTests::test_container: ok")
