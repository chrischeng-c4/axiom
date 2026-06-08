# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_ndbm"
# dimension = "behavior"
# case = "dbm_test_case__test_nonascii_filename"
# subject = "cpython.test_dbm_ndbm.DbmTestCase.test_nonascii_filename"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_ndbm.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: DbmTestCase::test_nonascii_filename (CPython 3.12 oracle)."""

import dbm.ndbm
import os
import tempfile


with tempfile.TemporaryDirectory() as tmpdir:
    filename = os.path.join(tmpdir, "test_dbm_é_🐍")
    with dbm.ndbm.open(filename, "c") as db:
        db[b"key"] = b"value"
    assert any(os.path.exists(filename + suffix) for suffix in ["", ".pag", ".dir", ".db"])
    with dbm.ndbm.open(filename, "r") as db:
        assert list(db.keys()) == [b"key"]
        assert b"key" in db
        assert db[b"key"] == b"value"

print("DbmTestCase::test_nonascii_filename: ok")
