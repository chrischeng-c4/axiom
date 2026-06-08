# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_disallow_instantiation"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_disallow_instantiation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestGdbm::test_disallow_instantiation (CPython 3.12 oracle)."""

import dbm.gnu as gdbm
import tempfile


with tempfile.TemporaryDirectory() as tmpdir:
    filename = f"{tmpdir}/test_gdbm"
    db = gdbm.open(filename, "c")
    try:
        db_type = type(db)
        try:
            db_type()
        except TypeError as exc:
            assert str(exc) == "cannot create '_gdbm.gdbm' instances", str(exc)
        else:
            raise AssertionError("_gdbm.gdbm should reject direct instantiation")
    finally:
        db.close()

print("TestGdbm::test_disallow_instantiation: ok")
