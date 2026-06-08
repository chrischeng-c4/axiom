# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "data_persists_across_reopen"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: data written under mode 'c' survives close and is readable after reopen in mode 'r'"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["persist_key"] = "persist_value"
    # Reopen and verify.
    with dbm.open(_path, "r") as _db:
        assert "persist_key" in _db, "key persisted"
        assert _db["persist_key"] == b"persist_value", f"value persisted: {_db['persist_key']!r}"
print("data_persists_across_reopen OK")
