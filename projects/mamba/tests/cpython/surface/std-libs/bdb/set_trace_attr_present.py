# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "surface"
# case = "set_trace_attr_present"
# subject = "bdb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb: set_trace_attr_present (surface)."""
import bdb

assert hasattr(bdb, "set_trace")
print("set_trace_attr_present OK")
