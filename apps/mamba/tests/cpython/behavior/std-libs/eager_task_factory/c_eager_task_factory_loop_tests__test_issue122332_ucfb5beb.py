# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "eager_task_factory"
# dimension = "behavior"
# case = "c_eager_task_factory_loop_tests__test_issue122332_ucfb5beb"
# subject = "cpython.test_eager_task_factory.CEagerTaskFactoryLoopTests.test_issue122332"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_eager_task_factory.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_eager_task_factory
_suite = unittest.defaultTestLoader.loadTestsFromName("CEagerTaskFactoryLoopTests.test_issue122332", test_eager_task_factory)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CEagerTaskFactoryLoopTests.test_issue122332 did not pass"
print("CEagerTaskFactoryLoopTests::test_issue122332: ok")
