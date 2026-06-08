# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "cursor_factory_tests__test_invalid_factory_ucb0f0f6"
# subject = "cpython.test_factory.CursorFactoryTests.test_invalid_factory"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_factory.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sqlite3 as sqlite
from collections.abc import Sequence
self_con = sqlite.connect(':memory:')
try:
    self_con.cursor(None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    self_con.cursor(lambda: None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    self_con.cursor(lambda con: None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("CursorFactoryTests::test_invalid_factory: ok")
