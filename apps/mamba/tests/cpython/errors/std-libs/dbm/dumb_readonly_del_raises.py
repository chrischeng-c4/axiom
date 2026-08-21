# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_readonly_del_raises"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: deleting through a read-only dumb handle raises dbm.dumb.error with the fixed 'opened for reading only' message"""
import dbm.dumb
import os
import tempfile

_ro_msg = "The database is opened for reading only"
with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.dumb.open(_path, "c") as _w:
        _w[b"a"] = b"1"
    with dbm.dumb.open(_path, "r") as _r:
        _raised = False
        try:
            del _r[b"a"]
        except dbm.dumb.error as e:
            _raised = True
            assert str(e) == _ro_msg, f"ro del msg = {str(e)!r}"
        assert _raised, "delete on read-only db must raise"
print("dumb_readonly_del_raises OK")
