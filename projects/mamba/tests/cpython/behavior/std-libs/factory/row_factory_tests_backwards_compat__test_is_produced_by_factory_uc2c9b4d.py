# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "row_factory_tests_backwards_compat__test_is_produced_by_factory_uc2c9b4d"
# subject = "cpython.test_factory.RowFactoryTestsBackwardsCompat.test_is_produced_by_factory"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_factory.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_factory
_suite = unittest.defaultTestLoader.loadTestsFromName("RowFactoryTestsBackwardsCompat.test_is_produced_by_factory", test_factory)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RowFactoryTestsBackwardsCompat.test_is_produced_by_factory did not pass"
print("RowFactoryTestsBackwardsCompat::test_is_produced_by_factory: ok")
