# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "setdefault_like_dict"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: setdefault stores and returns the default for a new key, and keeps the existing value on a second call"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _r = _db.setdefault(b"xxx", b"foo")
        assert _r == b"foo", f"setdefault new returns default = {_r!r}"
        assert _db[b"xxx"] == b"foo", "setdefault stored value"
        # A second call leaves the existing value untouched.
        _r2 = _db.setdefault(b"xxx", b"other")
        assert _r2 == b"foo", f"setdefault keeps existing = {_r2!r}"
print("setdefault_like_dict OK")
