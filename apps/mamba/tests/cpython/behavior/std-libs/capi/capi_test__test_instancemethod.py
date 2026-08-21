# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "capi"
# dimension = "behavior"
# case = "capi_test__test_instancemethod"
# subject = "cpython.test_capi.test_misc.CAPITest.test_instancemethod"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CPython C API instance-method wrapper behavior matches test_capi."""

import io
import unittest
from test.test_capi import test_misc


stream = io.StringIO()
suite = unittest.TestSuite([test_misc.CAPITest("test_instancemethod")])
result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("capi_test__test_instancemethod skipped:", result.skipped[0][1])
else:
    print("capi_test__test_instancemethod OK")
