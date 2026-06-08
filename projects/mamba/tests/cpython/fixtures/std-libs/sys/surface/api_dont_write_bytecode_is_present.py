# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_dont_write_bytecode_is_present"
# subject = "sys.dont_write_bytecode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.dont_write_bytecode: api_dont_write_bytecode_is_present (surface)."""
import sys

assert hasattr(sys, "dont_write_bytecode")
print("api_dont_write_bytecode_is_present OK")
