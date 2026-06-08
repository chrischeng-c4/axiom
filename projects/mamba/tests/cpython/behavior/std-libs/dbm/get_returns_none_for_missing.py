# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "get_returns_none_for_missing"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: get returns the stored value for a present key, None for an absent key, and the supplied default otherwise"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["present"] = "yes"
        assert _db.get("present") == b"yes", "get present"
        assert _db.get("absent") is None, "get absent is None"
        assert _db.get("absent", b"fallback") == b"fallback", "get absent default"
print("get_returns_none_for_missing OK")
