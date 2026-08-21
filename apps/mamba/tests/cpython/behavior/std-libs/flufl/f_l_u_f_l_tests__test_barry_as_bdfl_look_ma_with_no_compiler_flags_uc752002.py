# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "flufl"
# dimension = "behavior"
# case = "f_l_u_f_l_tests__test_barry_as_bdfl_look_ma_with_no_compiler_flags_uc752002"
# subject = "cpython.test_flufl.FLUFLTests.test_barry_as_bdfl_look_ma_with_no_compiler_flags"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_flufl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_flufl
_suite = unittest.defaultTestLoader.loadTestsFromName("FLUFLTests.test_barry_as_bdfl_look_ma_with_no_compiler_flags", test_flufl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FLUFLTests.test_barry_as_bdfl_look_ma_with_no_compiler_flags did not pass"
print("FLUFLTests::test_barry_as_bdfl_look_ma_with_no_compiler_flags: ok")
