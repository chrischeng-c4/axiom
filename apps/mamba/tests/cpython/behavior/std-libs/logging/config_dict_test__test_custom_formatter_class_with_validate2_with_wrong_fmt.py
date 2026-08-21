# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "config_dict_test__test_custom_formatter_class_with_validate2_with_wrong_fmt"
# subject = "cpython.test_logging.ConfigDictTest.test_custom_formatter_class_with_validate2_with_wrong_fmt"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("ConfigDictTest.test_custom_formatter_class_with_validate2_with_wrong_fmt", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConfigDictTest.test_custom_formatter_class_with_validate2_with_wrong_fmt did not pass"
print("ConfigDictTest::test_custom_formatter_class_with_validate2_with_wrong_fmt: ok")
