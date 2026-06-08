# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "version_is_str"
# subject = "sys.version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.version: version_is_str (surface)."""
import sys

assert type(sys.version).__name__ == "str"
print("version_is_str OK")
