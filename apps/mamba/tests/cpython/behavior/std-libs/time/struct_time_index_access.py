# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "struct_time_index_access"
# subject = "time.struct_time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.struct_time: struct_time supports integer index access mirroring the named fields: gmtime(0)[0]==1970, [1]==1, [5]==0"""
import time

_st = time.gmtime(0)
assert _st[0] == 1970, f"struct_time[0] = {_st[0]!r}"
assert _st[1] == 1, f"struct_time[1] = {_st[1]!r}"
assert _st[5] == 0, f"struct_time[5] = {_st[5]!r}"
print("struct_time_index_access OK")
