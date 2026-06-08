# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "str_and_bytes_keys_same_entry"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: a str-written key is reachable via its bytes form, and keys() reports the bytes spelling"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["1"] = "a"            # str key, str value
    with dbm.open(_path, "r") as _db:
        # The same entry is reachable via the bytes form of the key.
        assert _db[b"1"] == b"a", f"str-write/bytes-read = {_db[b'1']!r}"
        assert b"1" in list(_db.keys()), "keys() reports bytes form"
print("str_and_bytes_keys_same_entry OK")
