# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_ndbm"
# dimension = "behavior"
# case = "dbm_test_case__test_empty_value"
# subject = "cpython.test_dbm_ndbm.DbmTestCase.test_empty_value"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_ndbm.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: DbmTestCase::test_empty_value (CPython 3.12 oracle)."""

import dbm.ndbm
import os
import tempfile


if dbm.ndbm.library == "Berkeley DB":
    print("DbmTestCase::test_empty_value skipped: Berkeley DB empty value semantics")
else:
    with tempfile.TemporaryDirectory() as tmpdir:
        filename = os.path.join(tmpdir, "test_ndbm")
        with dbm.ndbm.open(filename, "c") as db:
            assert db.keys() == []
            db["empty"] = ""
            assert db.keys() == [b"empty"]
            assert b"empty" in db
            assert db[b"empty"] == b""
            assert db.get(b"empty") == b""
            assert db.setdefault(b"empty") == b""

    print("DbmTestCase::test_empty_value: ok")
