# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "whichdb_empty_db_file_is_none"
# subject = "dbm.whichdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.whichdb: whichdb returns None for a non-existent path and for a bare empty .db file (issue 17198)"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    # A non-existent path is recognized by nobody.
    _missing = os.path.join(_d, "missing")
    assert dbm.whichdb(_missing) is None, "whichdb(missing) is None"

    # A bare, empty `.db` file is not a real database -> None (issue 17198:
    # whichdb must not misreport an empty .db as the ndbm backend).
    _bare = os.path.join(_d, "bare")
    with open(_bare + ".db", "wb"):
        pass
    assert dbm.whichdb(_bare) is None, f"empty .db = {dbm.whichdb(_bare)!r}"
    assert dbm.whichdb(os.fsencode(_bare)) is None, "empty .db (bytes path)"
print("whichdb_empty_db_file_is_none OK")
