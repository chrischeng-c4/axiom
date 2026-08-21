# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_xxinterpchannels"
# dimension = "behavior"
# case = "exhaustive_channel_tests__test_force_close_uca2c071"
# subject = "cpython.test__xxinterpchannels.ExhaustiveChannelTests.test_force_close"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__xxinterpchannels.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test__xxinterpchannels
_suite = unittest.defaultTestLoader.loadTestsFromName("ExhaustiveChannelTests.test_force_close", test__xxinterpchannels)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExhaustiveChannelTests.test_force_close did not pass"
print("ExhaustiveChannelTests::test_force_close: ok")
