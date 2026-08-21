# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "typecodes_is_str_attr"
# subject = "array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array: typecodes_is_str_attr (surface)."""
import array

assert hasattr(array, "typecodes")
print("typecodes_is_str_attr OK")
