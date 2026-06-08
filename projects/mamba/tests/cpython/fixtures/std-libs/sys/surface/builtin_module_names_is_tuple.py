# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "builtin_module_names_is_tuple"
# subject = "sys.builtin_module_names"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.builtin_module_names: builtin_module_names_is_tuple (surface)."""
import sys

assert type(sys.builtin_module_names).__name__ == "tuple"
print("builtin_module_names_is_tuple OK")
