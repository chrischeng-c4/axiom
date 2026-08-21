# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "utf8_keys_values_roundtrip"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: a non-ASCII (UTF-8) str key round-trips and is readable via its UTF-8-encoded bytes form"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _ukey = "ü"  # u-umlaut
    with dbm.open(_path, "c") as _db:
        _db[_ukey] = b"!"
    with dbm.open(_path, "r") as _db:
        assert _ukey in _db, "non-ASCII str key present"
        assert _db[_ukey.encode("utf-8")] == b"!", "non-ASCII bytes key reads"
print("utf8_keys_values_roundtrip OK")
