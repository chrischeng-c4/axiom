# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "path_entries_are_str"
# subject = "sys.path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.path: every entry of sys.path is a str"""
import sys

assert all(isinstance(p, str) for p in sys.path), "all path entries are str"
print("path_entries_are_str OK")
