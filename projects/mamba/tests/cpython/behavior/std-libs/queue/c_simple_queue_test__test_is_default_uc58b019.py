# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "behavior"
# case = "c_simple_queue_test__test_is_default_uc58b019"
# subject = "cpython.test_queue.CSimpleQueueTest.test_is_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_queue
_suite = unittest.defaultTestLoader.loadTestsFromName("CSimpleQueueTest.test_is_default", test_queue)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CSimpleQueueTest.test_is_default did not pass"
print("CSimpleQueueTest::test_is_default: ok")
