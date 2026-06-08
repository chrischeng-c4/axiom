# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "set_break_invalid_file_returns_error_string"
# subject = "bdb.Bdb.set_break"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb() is dict-like, no set_break method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.set_break: set_break on a nonexistent file returns an error string rather than raising"""
import bdb

_d = bdb.Bdb()
_err = _d.set_break("/no/such/file.py", 1)
assert isinstance(_err, str), f"set_break invalid returns a str, got {_err!r}"
assert _err, "error message is non-empty"

print("set_break_invalid_file_returns_error_string OK")
