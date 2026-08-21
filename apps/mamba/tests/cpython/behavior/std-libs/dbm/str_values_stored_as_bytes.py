# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "str_values_stored_as_bytes"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: str inputs are stored and read back as bytes; bytes keys/values round-trip unchanged"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["key"] = "value"
        _v = _db["key"]
        assert isinstance(_v, bytes), f"value is bytes: {type(_v)!r}"
        assert _v == b"value", f"value = {_v!r}"
        _db[b"bkey"] = b"bval"
        assert _db[b"bkey"] == b"bval", "bytes key/val"
print("str_values_stored_as_bytes OK")
