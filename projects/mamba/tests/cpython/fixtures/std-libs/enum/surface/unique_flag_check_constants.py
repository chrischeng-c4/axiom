# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "unique_flag_check_constants"
# subject = "enum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum: unique_flag_check_constants (surface)."""
import enum

assert hasattr(enum, "UNIQUE")
print("unique_flag_check_constants OK")
