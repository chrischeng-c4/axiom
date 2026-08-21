# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "clear_break_no_breakpoints_returns_error_string"
# subject = "bdb.Bdb.clear_break"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb() is dict-like, no clear_break method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.clear_break: clear_break where no breakpoint exists returns an error string rather than raising"""
import bdb

_d = bdb.Bdb()
_err = _d.clear_break("/some/file.py", 999)
assert isinstance(_err, str), f"clear_break invalid returns a str, got {_err!r}"
assert _err, "error message is non-empty"

print("clear_break_no_breakpoints_returns_error_string OK")
