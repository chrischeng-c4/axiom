# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tasks"
# dimension = "behavior"
# case = "c_task__c_future__tests__test_del__log_destroy_pending_segfault"
# subject = "cpython.test_tasks.CTask_CFuture_Tests.test_del__log_destroy_pending_segfault"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_tasks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_tasks
_suite = unittest.defaultTestLoader.loadTestsFromName("CTask_CFuture_Tests.test_del__log_destroy_pending_segfault", test_tasks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CTask_CFuture_Tests.test_del__log_destroy_pending_segfault did not pass"
print("CTask_CFuture_Tests::test_del__log_destroy_pending_segfault: ok")
