# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "row_factory_tests__test_fake_cursor_class_uc637c9a"
# subject = "cpython.test_factory.RowFactoryTests.test_fake_cursor_class"
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
self_con.row_factory = sqlite.Row

class FakeCursor(str):
    __class__ = sqlite.Cursor
try:
    self_con.cursor(FakeCursor)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    sqlite.Row(FakeCursor(), ())
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("RowFactoryTests::test_fake_cursor_class: ok")
