# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "peepholer"
# dimension = "behavior"
# case = "direct_cfg_optimizer_tests__test_no_unsafe_static_swap"
# subject = "cpython.test_peepholer.DirectCfgOptimizerTests.test_no_unsafe_static_swap"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_peepholer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_peepholer
_suite = unittest.defaultTestLoader.loadTestsFromName("DirectCfgOptimizerTests.test_no_unsafe_static_swap", test_peepholer)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DirectCfgOptimizerTests.test_no_unsafe_static_swap did not pass"
print("DirectCfgOptimizerTests::test_no_unsafe_static_swap: ok")
