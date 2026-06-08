# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "items_yields_all_pairs"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: items() yields every stored (key, value) pair as bytes (dumb backend)"""
import dbm.dumb
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.dumb.open(_path, "c") as _db:
        _db[b"a"] = b"1"
        _db[b"b"] = b"2"
    with dbm.dumb.open(_path, "r") as _db:
        assert sorted(_db.items()) == [(b"a", b"1"), (b"b", b"2")], \
            f"items round-trip = {sorted(_db.items())!r}"
print("items_yields_all_pairs OK")
