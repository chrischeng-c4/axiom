# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "commands_with_spaces__test_noshell_string_with_spaces"
# subject = "cpython.test_subprocess.CommandsWithSpaces.test_noshell_string_with_spaces"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("CommandsWithSpaces.test_noshell_string_with_spaces", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CommandsWithSpaces.test_noshell_string_with_spaces did not pass"
print("CommandsWithSpaces::test_noshell_string_with_spaces: ok")
