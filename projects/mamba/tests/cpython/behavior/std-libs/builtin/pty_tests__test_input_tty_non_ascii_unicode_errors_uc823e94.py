# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtin"
# dimension = "behavior"
# case = "pty_tests__test_input_tty_non_ascii_unicode_errors_uc823e94"
# subject = "cpython.test_builtin.PtyTests.test_input_tty_non_ascii_unicode_errors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_builtin
_suite = unittest.defaultTestLoader.loadTestsFromName("PtyTests.test_input_tty_non_ascii_unicode_errors", test_builtin)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PtyTests.test_input_tty_non_ascii_unicode_errors did not pass"
print("PtyTests::test_input_tty_non_ascii_unicode_errors: ok")
