# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "non_a_s_c_i_i_tests__test_metadata_loads_egg_info"
# subject = "cpython.test_main.NonASCIITests.test_metadata_loads_egg_info"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("NonASCIITests.test_metadata_loads_egg_info", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NonASCIITests.test_metadata_loads_egg_info did not pass"
print("NonASCIITests::test_metadata_loads_egg_info: ok")
