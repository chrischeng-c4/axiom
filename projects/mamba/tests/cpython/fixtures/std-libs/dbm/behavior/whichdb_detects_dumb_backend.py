# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "whichdb_detects_dumb_backend"
# subject = "dbm.whichdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.whichdb: whichdb identifies a dumb-format db as 'dbm.dumb' for both str and bytes path spellings, even when empty"""
import dbm
import dbm.dumb
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _dbpath = os.path.join(_d, "store")
    # An empty dumb db is still detected as 'dbm.dumb'.
    with dbm.dumb.open(_dbpath, "c") as _f:
        pass
    assert dbm.whichdb(_dbpath) == "dbm.dumb", f"empty dumb db = {dbm.whichdb(_dbpath)!r}"

    with dbm.dumb.open(_dbpath, "w") as _f:
        _f[b"key"] = b"value"
    # The answer is independent of how the path is spelled (str or bytes).
    for _path in (_dbpath, os.fsencode(_dbpath)):
        assert dbm.whichdb(_path) == "dbm.dumb", \
            f"populated dumb db ({type(_path).__name__}) = {dbm.whichdb(_path)!r}"
print("whichdb_detects_dumb_backend OK")
