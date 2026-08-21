# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cli"
# dimension = "behavior"
# case = "command_line_interface__test_cli_on_disk_db_uc1a6bc7"
# subject = "cpython.test_cli.CommandLineInterface.test_cli_on_disk_db"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_cli.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_cli
_suite = unittest.defaultTestLoader.loadTestsFromName("CommandLineInterface.test_cli_on_disk_db", test_cli)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CommandLineInterface.test_cli_on_disk_db did not pass"
print("CommandLineInterface::test_cli_on_disk_db: ok")
