# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "meta_path_unique_finders"
# subject = "sys.meta_path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.meta_path: sys.meta_path is a list of import finders with no duplicate entries"""
import sys

assert isinstance(sys.meta_path, list), f"meta_path type = {type(sys.meta_path)!r}"
assert len(sys.meta_path) == len(set(sys.meta_path)), "meta_path has no duplicates"
print("meta_path_unique_finders OK")
