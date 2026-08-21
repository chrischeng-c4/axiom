# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "struct_time_is_callable"
# subject = "time.struct_time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.struct_time: struct_time_is_callable (surface)."""
import time

assert callable(time.struct_time)
print("struct_time_is_callable OK")
