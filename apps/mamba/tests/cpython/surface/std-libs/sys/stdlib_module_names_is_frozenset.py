# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "stdlib_module_names_is_frozenset"
# subject = "sys.stdlib_module_names"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.stdlib_module_names: stdlib_module_names_is_frozenset (surface)."""
import sys

assert type(sys.stdlib_module_names).__name__ == "frozenset"
print("stdlib_module_names_is_frozenset OK")
