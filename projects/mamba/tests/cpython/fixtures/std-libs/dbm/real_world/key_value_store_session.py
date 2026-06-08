# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "real_world"
# case = "key_value_store_session"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: a small persistent key-value session: open 'c', write user records, close, reopen 'r', read back, update one record, and confirm it persists"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "sessions")

    # 1. Open for create, write a few user-session records, then close.
    _records = {
        "user:1001": "name=alice;role=admin",
        "user:1002": "name=bob;role=editor",
        "user:1003": "name=carol;role=viewer",
    }
    with dbm.open(_path, "c") as _db:
        for _k, _v in _records.items():
            _db[_k] = _v

    # 2. Reopen read-only and confirm every record reads back.
    with dbm.open(_path, "r") as _db:
        assert len(_db) == 3, f"three records persisted = {len(_db)!r}"
        assert _db["user:1001"] == b"name=alice;role=admin", "alice read back"
        assert _db["user:1003"] == b"name=carol;role=viewer", "carol read back"

    # 3. Reopen for write to promote one user, then confirm the change persists.
    with dbm.open(_path, "w") as _db:
        _db["user:1003"] = "name=carol;role=editor"
    with dbm.open(_path, "r") as _db:
        assert _db["user:1003"] == b"name=carol;role=editor", "carol promoted"
        assert _db["user:1001"] == b"name=alice;role=admin", "alice untouched"
        assert len(_db) == 3, "still three records"

print("key_value_store_session OK")
