# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tabnanny"
# dimension = "behavior"
# case = "test_command_line__test_verbose_mode_uc9dca15"
# subject = "cpython.test_tabnanny.TestCommandLine.test_verbose_mode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tabnanny.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tabnanny
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCommandLine.test_verbose_mode", test_tabnanny)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCommandLine.test_verbose_mode did not pass"
print("TestCommandLine::test_verbose_mode: ok")
