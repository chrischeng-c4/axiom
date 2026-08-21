# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "config_dict_test__test_listen_config_10_ok"
# subject = "cpython.test_logging.ConfigDictTest.test_listen_config_10_ok"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("ConfigDictTest.test_listen_config_10_ok", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConfigDictTest.test_listen_config_10_ok did not pass"
print("ConfigDictTest::test_listen_config_10_ok: ok")
