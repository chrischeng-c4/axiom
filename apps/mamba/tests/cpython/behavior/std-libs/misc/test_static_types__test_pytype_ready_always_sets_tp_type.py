# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "test_static_types__test_pytype_ready_always_sets_tp_type"
# subject = "cpython.test_misc.TestStaticTypes.test_pytype_ready_always_sets_tp_type"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("TestStaticTypes.test_pytype_ready_always_sets_tp_type", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestStaticTypes.test_pytype_ready_always_sets_tp_type did not pass"
print("TestStaticTypes::test_pytype_ready_always_sets_tp_type: ok")
