# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "overwrite_replaces_value"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: overwriting a key replaces its value and keeps the db at one key (CPython bug #482460)"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db[b"1"] = b"hello"
        _db[b"1"] = b"hello2"
    with dbm.open(_path, "r") as _db:
        assert _db[b"1"] == b"hello2", f"overwrite wins = {_db[b'1']!r}"
        assert len(_db) == 1, f"overwrite keeps one key = {len(_db)!r}"
print("overwrite_replaces_value OK")
