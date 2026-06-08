# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "named_flags_check_constant"
# subject = "enum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum: named_flags_check_constant (surface)."""
import enum

assert hasattr(enum, "NAMED_FLAGS")
print("named_flags_check_constant OK")
