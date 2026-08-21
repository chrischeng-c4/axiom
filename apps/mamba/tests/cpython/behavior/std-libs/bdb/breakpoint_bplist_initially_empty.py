# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "breakpoint_bplist_initially_empty"
# subject = "bdb.Breakpoint"
# kind = "semantic"
# xfail = "mamba bdb stub: Breakpoint.bplist is None, not an empty dict (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Breakpoint: the class-level Breakpoint.bplist registry is an empty dict before any breakpoint is created"""
import bdb

assert isinstance(bdb.Breakpoint.bplist, dict), f"bplist type = {type(bdb.Breakpoint.bplist)!r}"
assert bdb.Breakpoint.bplist == {}, f"bplist not empty: {bdb.Breakpoint.bplist!r}"

print("breakpoint_bplist_initially_empty OK")
