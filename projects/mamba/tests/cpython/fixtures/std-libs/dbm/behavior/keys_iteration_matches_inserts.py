# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "keys_iteration_matches_inserts"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: keys() after reopen reports exactly the inserted keys (as bytes), regardless of order"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _data = {"alpha": "A", "beta": "B", "gamma": "C"}
    with dbm.open(_path, "c") as _db:
        for _k, _v in _data.items():
            _db[_k] = _v
    with dbm.open(_path, "r") as _db:
        _keys = set(_db.keys())
        assert len(_keys) == 3, f"three keys = {len(_keys)!r}"
        for _k in _data:
            assert _k.encode() in _keys, f"{_k!r} in keys"
print("keys_iteration_matches_inserts OK")
