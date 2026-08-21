# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "continuous_check_constant"
# subject = "enum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum: continuous_check_constant (surface)."""
import enum

assert hasattr(enum, "CONTINUOUS")
print("continuous_check_constant OK")
