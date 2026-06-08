# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "config_dict_test__test_config_queue_handler_does_not_create_multiprocessing_manager"
# subject = "cpython.test_logging.ConfigDictTest.test_config_queue_handler_does_not_create_multiprocessing_manager"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("ConfigDictTest.test_config_queue_handler_does_not_create_multiprocessing_manager", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConfigDictTest.test_config_queue_handler_does_not_create_multiprocessing_manager did not pass"
print("ConfigDictTest::test_config_queue_handler_does_not_create_multiprocessing_manager: ok")
