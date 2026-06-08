# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "del_removes_key"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: del db[key] removes only that key; the remaining keys stay present"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["a"] = "1"
        _db["b"] = "2"
        del _db["a"]
        assert "a" not in _db, "a deleted"
        assert "b" in _db, "b remains"
print("del_removes_key OK")
