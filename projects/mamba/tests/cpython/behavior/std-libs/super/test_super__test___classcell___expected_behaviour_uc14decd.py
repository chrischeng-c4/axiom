# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "super"
# dimension = "behavior"
# case = "test_super__test___classcell___expected_behaviour_uc14decd"
# subject = "cpython.test_super.TestSuper.test___classcell___expected_behaviour"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_super.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_super
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSuper.test___classcell___expected_behaviour", test_super)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSuper.test___classcell___expected_behaviour did not pass"
print("TestSuper::test___classcell___expected_behaviour: ok")
