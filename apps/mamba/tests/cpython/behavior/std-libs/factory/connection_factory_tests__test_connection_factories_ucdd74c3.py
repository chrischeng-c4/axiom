# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "connection_factory_tests__test_connection_factories_ucdd74c3"
# subject = "cpython.test_factory.ConnectionFactoryTests.test_connection_factories"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_factory.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_factory
_suite = unittest.defaultTestLoader.loadTestsFromName("ConnectionFactoryTests.test_connection_factories", test_factory)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConnectionFactoryTests.test_connection_factories did not pass"
print("ConnectionFactoryTests::test_connection_factories: ok")
