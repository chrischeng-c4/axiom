# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "set_break_clear_all_roundtrip"
# subject = "bdb.Bdb.set_break"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb() is dict-like, no breaks/set_break/clear_all_breaks (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.set_break: breaks starts empty, set_break on a real source line populates it, and clear_all_breaks empties it again"""
import bdb
import os
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _src = os.path.join(_td, "module_under_debug.py")
    with open(_src, "w", encoding="utf-8") as _f:
        _f.write("def g():\n    return 1\n")

    _d = bdb.Bdb()
    assert _d.breaks == {}, "breaks empty initially"
    _err = _d.set_break(_src, 2)
    assert _err is None, f"set_break on a real line returns None, got {_err!r}"
    assert len(_d.breaks) > 0, "break added to breaks dict"
    _d.clear_all_breaks()
    assert _d.breaks == {}, "breaks cleared"

print("set_break_clear_all_roundtrip OK")
