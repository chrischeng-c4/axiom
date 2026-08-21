# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "get_break_missing_returns_false"
# subject = "bdb.Bdb.get_break"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb() is dict-like, no get_break method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.get_break: get_break for a file/line with no breakpoint returns False (no raise)"""
import bdb

_d = bdb.Bdb()
_res = _d.get_break("no_such_file.py", 1)
assert _res is False, f"get_break missing = {_res!r}"

print("get_break_missing_returns_false OK")
