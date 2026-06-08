# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "cursor_factory_tests__test_is_instance_uc778f29"
# subject = "cpython.test_factory.CursorFactoryTests.test_is_instance"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_factory.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_factory
_suite = unittest.defaultTestLoader.loadTestsFromName("CursorFactoryTests.test_is_instance", test_factory)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CursorFactoryTests.test_is_instance did not pass"
print("CursorFactoryTests::test_is_instance: ok")
