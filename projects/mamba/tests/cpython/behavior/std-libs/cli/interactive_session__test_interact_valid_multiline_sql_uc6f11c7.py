# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cli"
# dimension = "behavior"
# case = "interactive_session__test_interact_valid_multiline_sql_uc6f11c7"
# subject = "cpython.test_cli.InteractiveSession.test_interact_valid_multiline_sql"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_cli.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_cli
_suite = unittest.defaultTestLoader.loadTestsFromName("InteractiveSession.test_interact_valid_multiline_sql", test_cli)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InteractiveSession.test_interact_valid_multiline_sql did not pass"
print("InteractiveSession::test_interact_valid_multiline_sql: ok")
