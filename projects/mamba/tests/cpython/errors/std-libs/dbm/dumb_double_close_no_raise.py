# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_double_close_no_raise"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: closing an already-closed dumb handle a second time is a no-op and does not raise"""
import dbm.dumb
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _f = dbm.dumb.open(_path, "c")
    _f.close()
    # Double close is a no-op (must not raise).
    _f.close()
print("dumb_double_close_no_raise OK")
