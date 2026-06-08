# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "meta_path_is_list"
# subject = "sys.meta_path"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.meta_path: meta_path_is_list (surface)."""
import sys

assert type(sys.meta_path).__name__ == "list"
print("meta_path_is_list OK")
