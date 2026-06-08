# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_version_is_int"
# subject = "sys.api_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.api_version: api_version_is_int (surface)."""
import sys

assert type(sys.api_version).__name__ == "int"
print("api_version_is_int OK")
