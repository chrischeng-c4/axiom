# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "write_to_readonly_raises"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: writing a key to a db opened in read mode ('r') raises"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["init"] = "data"
    with dbm.open(_path, "r") as _db:
        _raised = False
        try:
            _db["new"] = "value"
        except Exception:
            _raised = True
        assert _raised, "write to read-only db raises"
print("write_to_readonly_raises OK")
