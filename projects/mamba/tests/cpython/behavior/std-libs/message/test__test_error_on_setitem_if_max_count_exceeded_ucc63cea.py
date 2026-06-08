# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "message"
# dimension = "behavior"
# case = "test__test_error_on_setitem_if_max_count_exceeded_ucc63cea"
# subject = "cpython.test_message.Test.test_error_on_setitem_if_max_count_exceeded"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_message.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_message
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_error_on_setitem_if_max_count_exceeded", test_message)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_error_on_setitem_if_max_count_exceeded did not pass"
print("Test::test_error_on_setitem_if_max_count_exceeded: ok")
