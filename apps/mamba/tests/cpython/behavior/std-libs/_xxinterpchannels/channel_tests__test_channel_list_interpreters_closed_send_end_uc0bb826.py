# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_xxinterpchannels"
# dimension = "behavior"
# case = "channel_tests__test_channel_list_interpreters_closed_send_end_uc0bb826"
# subject = "cpython.test__xxinterpchannels.ChannelTests.test_channel_list_interpreters_closed_send_end"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__xxinterpchannels.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test__xxinterpchannels
_suite = unittest.defaultTestLoader.loadTestsFromName("ChannelTests.test_channel_list_interpreters_closed_send_end", test__xxinterpchannels)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ChannelTests.test_channel_list_interpreters_closed_send_end did not pass"
print("ChannelTests::test_channel_list_interpreters_closed_send_end: ok")
