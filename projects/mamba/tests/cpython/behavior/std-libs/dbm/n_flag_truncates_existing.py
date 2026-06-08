# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "n_flag_truncates_existing"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: opening an existing db with the 'n' flag truncates it to empty before new writes"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["old"] = "data"
    # Opening with 'n' truncates the existing db.
    with dbm.open(_path, "n") as _db:
        assert "old" not in _db, "old data gone after 'n' open"
        _db["new"] = "fresh"
    with dbm.open(_path, "r") as _db:
        assert "new" in _db, "new data present"
        assert "old" not in _db, "old data still gone"
print("n_flag_truncates_existing OK")
