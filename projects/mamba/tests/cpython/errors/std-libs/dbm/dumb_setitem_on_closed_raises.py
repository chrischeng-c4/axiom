# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_setitem_on_closed_raises"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: setitem on a closed dumb handle raises dbm.dumb.error with the fixed 'already been closed' message"""
import dbm.dumb
import os
import tempfile

_closed_msg = "DBM object has already been closed"
with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _f = dbm.dumb.open(_path, "c")
    _f.close()
    _raised = False
    try:
        _f[b"test"] = b"v"
    except dbm.dumb.error as e:
        _raised = True
        assert str(e) == _closed_msg, f"setitem closed msg = {str(e)!r}"
    assert _raised, "setitem on closed db must raise"
print("dumb_setitem_on_closed_raises OK")
