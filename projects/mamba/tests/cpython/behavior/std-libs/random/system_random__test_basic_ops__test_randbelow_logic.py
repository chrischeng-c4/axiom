# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random__test_basic_ops__test_randbelow_logic"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_randbelow_logic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_random
_suite = unittest.defaultTestLoader.loadTestsFromName("SystemRandom_TestBasicOps.test_randbelow_logic", test_random)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SystemRandom_TestBasicOps.test_randbelow_logic did not pass"
print("SystemRandom_TestBasicOps::test_randbelow_logic: ok")
