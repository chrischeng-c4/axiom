# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "peepholer"
# dimension = "behavior"
# case = "test_marking_variables_as_un_known__test_load_fast_unknown_because_del"
# subject = "cpython.test_peepholer.TestMarkingVariablesAsUnKnown.test_load_fast_unknown_because_del"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_peepholer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_peepholer
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMarkingVariablesAsUnKnown.test_load_fast_unknown_because_del", test_peepholer)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMarkingVariablesAsUnKnown.test_load_fast_unknown_because_del did not pass"
print("TestMarkingVariablesAsUnKnown::test_load_fast_unknown_because_del: ok")
