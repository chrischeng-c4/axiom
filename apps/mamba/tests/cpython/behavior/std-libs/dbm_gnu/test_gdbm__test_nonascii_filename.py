# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_nonascii_filename"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_nonascii_filename"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestGdbm::test_nonascii_filename (CPython 3.12 oracle)."""

import dbm.gnu as gdbm
import os
import tempfile


with tempfile.TemporaryDirectory() as tmpdir:
    filename = os.path.join(tmpdir, "test_dbm_é_🐍")
    with gdbm.open(filename, "c") as db:
        db[b"key"] = b"value"
    assert os.path.exists(filename)
    with gdbm.open(filename, "r") as db:
        assert list(db.keys()) == [b"key"]
        assert b"key" in db
        assert db[b"key"] == b"value"

print("TestGdbm::test_nonascii_filename: ok")
