# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "mro_test__test_tp_subclasses_cycle_in_update_slots"
# subject = "cpython.test_descr.MroTest.test_tp_subclasses_cycle_in_update_slots"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_descr
_suite = unittest.defaultTestLoader.loadTestsFromName("MroTest.test_tp_subclasses_cycle_in_update_slots", test_descr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MroTest.test_tp_subclasses_cycle_in_update_slots did not pass"
print("MroTest::test_tp_subclasses_cycle_in_update_slots: ok")
