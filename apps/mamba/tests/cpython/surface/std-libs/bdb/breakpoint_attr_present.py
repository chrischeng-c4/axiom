# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "surface"
# case = "breakpoint_attr_present"
# subject = "bdb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb: breakpoint_attr_present (surface)."""
import bdb

assert hasattr(bdb, "Breakpoint")
print("breakpoint_attr_present OK")
