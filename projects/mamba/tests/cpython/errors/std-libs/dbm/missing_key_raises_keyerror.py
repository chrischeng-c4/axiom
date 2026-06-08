# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "missing_key_raises_keyerror"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: indexing an absent key on an open db raises KeyError"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["exists"] = "yes"
        _raised = False
        try:
            _ = _db["missing"]
        except KeyError:
            _raised = True
        assert _raised, "missing key raises KeyError"
print("missing_key_raises_keyerror OK")
