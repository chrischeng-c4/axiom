# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "loading"
# dimension = "behavior"
# case = "loader_test__test_1703286_b_uc00efd4"
# subject = "cpython.test_loading.LoaderTest.test_1703286_B"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_loading.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_loading
_suite = unittest.defaultTestLoader.loadTestsFromName("LoaderTest.test_1703286_B", test_loading)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LoaderTest.test_1703286_B did not pass"
print("LoaderTest::test_1703286_B: ok")
