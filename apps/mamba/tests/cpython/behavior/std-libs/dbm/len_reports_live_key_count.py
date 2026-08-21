# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "len_reports_live_key_count"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: len(db) reflects the live key count across inserts and a delete"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        assert len(_db) == 0, f"empty len = {len(_db)!r}"
        _db["a"] = "1"
        _db["b"] = "2"
        assert len(_db) == 2, f"len after 2 inserts = {len(_db)!r}"
        del _db["a"]
        assert len(_db) == 1, f"len after del = {len(_db)!r}"
print("len_reports_live_key_count OK")
