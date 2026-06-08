# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "empty_bytes_value_distinct_from_absent"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: an empty-bytes value is stored and present, and stays distinguishable from an absent key (get -> None)"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db[b"empty"] = b""
        assert b"empty" in _db, "empty-valued key is present"
        assert _db[b"empty"] == b"", f"empty value = {_db[b'empty']!r}"
        assert _db.get(b"empty") == b"", "get returns empty bytes"
        # An absent key is still distinguishable from an empty value.
        assert _db.get(b"never") is None, "absent key still None"
print("empty_bytes_value_distinct_from_absent OK")
