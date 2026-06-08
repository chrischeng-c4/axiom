# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "peepholer"
# dimension = "behavior"
# case = "test_tranforms__test_static_swaps_unpack_three"
# subject = "cpython.test_peepholer.TestTranforms.test_static_swaps_unpack_three"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_peepholer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_peepholer
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTranforms.test_static_swaps_unpack_three", test_peepholer)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTranforms.test_static_swaps_unpack_three did not pass"
print("TestTranforms::test_static_swaps_unpack_three: ok")
