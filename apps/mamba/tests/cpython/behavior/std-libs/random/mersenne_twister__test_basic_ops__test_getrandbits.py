# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister__test_basic_ops__test_getrandbits"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_getrandbits"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_random
_suite = unittest.defaultTestLoader.loadTestsFromName("MersenneTwister_TestBasicOps.test_getrandbits", test_random)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MersenneTwister_TestBasicOps.test_getrandbits did not pass"
print("MersenneTwister_TestBasicOps::test_getrandbits: ok")
