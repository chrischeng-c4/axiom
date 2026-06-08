# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "whichdb_reports_module_name"
# subject = "dbm.whichdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.whichdb: whichdb returns a non-empty backend module name (str) for an existing db"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["x"] = "y"
    _which = dbm.whichdb(_path)
    assert _which is not None, f"whichdb returns module: {_which!r}"
    assert isinstance(_which, str), f"whichdb type = {type(_which)!r}"
    assert _which != "", "whichdb name non-empty"
print("whichdb_reports_module_name OK")
