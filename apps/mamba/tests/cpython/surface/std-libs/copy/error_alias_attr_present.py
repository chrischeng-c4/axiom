# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "surface"
# case = "error_alias_attr_present"
# subject = "copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""copy: error_alias_attr_present (surface)."""
import copy

assert hasattr(copy, "error")
print("error_alias_attr_present OK")
