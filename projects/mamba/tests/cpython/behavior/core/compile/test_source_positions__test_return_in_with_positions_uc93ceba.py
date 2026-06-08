# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_source_positions__test_return_in_with_positions_uc93ceba"
# subject = "cpython.test_compile.TestSourcePositions.test_return_in_with_positions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_compile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSourcePositions.test_return_in_with_positions", test_compile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSourcePositions.test_return_in_with_positions did not pass"
print("TestSourcePositions::test_return_in_with_positions: ok")
