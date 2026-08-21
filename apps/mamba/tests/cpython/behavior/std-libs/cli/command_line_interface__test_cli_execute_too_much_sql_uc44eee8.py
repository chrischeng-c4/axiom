# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cli"
# dimension = "behavior"
# case = "command_line_interface__test_cli_execute_too_much_sql_uc44eee8"
# subject = "cpython.test_cli.CommandLineInterface.test_cli_execute_too_much_sql"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_cli.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_cli
_suite = unittest.defaultTestLoader.loadTestsFromName("CommandLineInterface.test_cli_execute_too_much_sql", test_cli)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CommandLineInterface.test_cli_execute_too_much_sql did not pass"
print("CommandLineInterface::test_cli_execute_too_much_sql: ok")
